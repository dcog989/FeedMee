use crate::{
    AppState, db,
    models::{Article, Folder},
    settings::{self, AppSettings},
};
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use readability_rust::Readability;
use scraper::{Html, Selector};
use serde::Serialize;
use std::fmt::Write;

#[derive(Serialize)]
pub struct AppInfo {
    pub version: String,
    pub data_path: String,
    pub logs_path: String,
    pub db_path: String,
}

#[tauri::command]
pub fn get_app_info(app: tauri::AppHandle) -> Result<AppInfo, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let version = app.package_info().version.to_string();

    Ok(AppInfo {
        version,
        data_path: app_data_dir.to_string_lossy().to_string(),
        logs_path: app_data_dir.join("Logs").to_string_lossy().to_string(),
        db_path: app_data_dir.join("Database").join("feedmee.sqlite").to_string_lossy().to_string(),
    })
}
use std::io::Cursor;
use tauri::{AppHandle, Manager, State};
use url::Url;

fn create_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().unwrap();
    Ok(settings.clone())
}

#[tauri::command]
pub fn save_app_settings(
    new_settings: AppSettings,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut settings_guard = state.settings.lock().unwrap();
    *settings_guard = new_settings.clone();

    if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        settings::save_settings(&app_data_dir, &new_settings);
        info!("Settings saved to disk");
        Ok(())
    } else {
        Err("Could not determine app data directory".to_string())
    }
}

#[tauri::command]
pub fn get_folders_with_feeds(state: State<'_, AppState>) -> Result<Vec<Folder>, String> {
    let conn = state.db.lock().unwrap();
    db::get_folders_with_feeds(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_articles_for_feed(
    feed_id: i64,
    limit: usize,
    offset: usize,
    sort_desc: bool,
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    let conn = state.db.lock().unwrap();
    db::get_articles_for_feed(&conn, feed_id, limit, offset, !sort_desc).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_articles_for_folder(
    folder_id: i64,
    limit: usize,
    offset: usize,
    sort_desc: bool,
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    let conn = state.db.lock().unwrap();
    db::get_articles_for_folder(&conn, folder_id, limit, offset, !sort_desc)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_latest_articles(
    cutoff_timestamp: i64,
    limit: usize,
    offset: usize,
    sort_desc: bool,
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    let conn = state.db.lock().unwrap();
    db::get_latest_articles(&conn, cutoff_timestamp, limit, offset, !sort_desc)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_saved_articles(
    limit: usize,
    offset: usize,
    sort_desc: bool,
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    let conn = state.db.lock().unwrap();
    db::get_saved_articles(&conn, limit, offset, !sort_desc).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_folder(name: String, state: State<'_, AppState>) -> Result<i64, String> {
    let conn = state.db.lock().unwrap();
    db::create_folder(&conn, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_article_saved(
    id: i64,
    is_saved: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::update_article_saved(&conn, id, is_saved).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_article_read(id: i64, read: bool, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::set_article_read(&conn, id, read).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_all_read(
    target_type: String,
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Mark All Read: type={}, id={}", target_type, id);
    let conn = state.db.lock().unwrap();
    if target_type == "feed" {
        db::mark_feed_read(&conn, id).map_err(|e| e.to_string())
    } else if target_type == "folder" {
        db::mark_folder_read(&conn, id).map_err(|e| e.to_string())
    } else {
        Err("Invalid type".to_string())
    }
}

#[tauri::command]
pub async fn import_opml(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let xml_content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let document = opml::OPML::from_str(&xml_content).map_err(|e| e.to_string())?;
    let conn = state.db.lock().unwrap();
    let default_folder_id = db::create_folder(&conn, "Uncategorized").map_err(|e| e.to_string())?;

    for outline in document.body.outlines {
        if !outline.outlines.is_empty() {
            let folder_name = outline.text;
            if let Ok(folder_id) = db::create_folder(&conn, &folder_name) {
                for child in outline.outlines {
                    if let Some(url) = child.xml_url {
                        let _ = db::create_feed(&conn, &child.text, &url, folder_id);
                    }
                }
            }
        } else if let Some(url) = outline.xml_url {
            let _ = db::create_feed(&conn, &outline.text, &url, default_folder_id);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn export_opml(state: State<'_, AppState>) -> Result<String, String> {
    let folders = {
        let conn = state.db.lock().unwrap();
        db::get_folders_with_feeds(&conn).map_err(|e| e.to_string())?
    };

    let mut opml = String::new();
    writeln!(&mut opml, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>").unwrap();
    writeln!(&mut opml, "<opml version=\"2.0\">").unwrap();
    writeln!(&mut opml, "  <head><title>FeedMee Export</title></head>").unwrap();
    writeln!(&mut opml, "  <body>").unwrap();

    for folder in folders {
        if folder.feeds.is_empty() {
            continue;
        }
        let escaped_name = folder.name.replace("\"", "&quot;");
        writeln!(&mut opml, "    <outline text=\"{}\">", escaped_name).unwrap();
        for feed in &folder.feeds {
            let escaped_feed_name = feed.name.replace("\"", "&quot;");
            let escaped_url = feed.url.replace("\"", "&quot;");
            writeln!(
                &mut opml,
                "      <outline type=\"rss\" text=\"{}\" xmlUrl=\"{}\" />",
                escaped_feed_name, escaped_url
            )
            .unwrap();
        }
        writeln!(&mut opml, "    </outline>").unwrap();
    }
    writeln!(&mut opml, "  </body>").unwrap();
    writeln!(&mut opml, "</opml>").unwrap();
    Ok(opml)
}

#[tauri::command]
pub async fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_article_content(url: String) -> Result<String, String> {
    let client = create_client()?;
    let html = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let mut parser = Readability::new(&html, None).map_err(|e| format!("{:?}", e))?;
    let article = parser.parse().ok_or("Failed to parse content")?;
    article.content.ok_or("No content extracted".to_string())
}

#[tauri::command]
pub async fn refresh_feed(feed_id: i64, state: State<'_, AppState>) -> Result<usize, String> {
    let url = {
        let conn = state.db.lock().unwrap();
        db::get_feed_url(&conn, feed_id).map_err(|e| e.to_string())?
    };

    let client = create_client()?;
    let result = client.get(&url).send().await;

    match result {
        Ok(response) => {
            let content = response.bytes().await.map_err(|e| e.to_string())?;
            match feed_rs::parser::parse(Cursor::new(content)) {
                Ok(feed) => {
                    let conn = state.db.lock().unwrap();
                    let mut count = 0;
                    for entry in feed.entries {
                        let article_url = entry
                            .links
                            .iter()
                            .find(|l| l.rel.as_deref() == Some("alternate"))
                            .or(entry.links.first())
                            .map(|l| l.href.clone())
                            .unwrap_or_default();

                        let article = Article {
                            id: 0,
                            feed_id,
                            title: entry
                                .title
                                .map(|t| t.content)
                                .unwrap_or_else(|| "No Title".to_string()),
                            author: entry
                                .authors
                                .first()
                                .map(|p| p.name.clone())
                                .unwrap_or_default(),
                            summary: entry
                                .summary
                                .map(|s| s.content)
                                .or(entry.content.map(|c| c.body.unwrap_or_default()))
                                .unwrap_or_default(),
                            url: article_url,
                            timestamp: entry
                                .published
                                .or(entry.updated)
                                .map(|d| d.timestamp())
                                .unwrap_or(0),
                            is_read: false,
                            is_saved: false,
                        };
                        if !article.url.is_empty() && db::insert_article(&conn, &article).is_ok() {
                            count += 1;
                        }
                    }
                    let _ = db::update_feed_error(&conn, feed_id, false);
                    Ok(count)
                },
                Err(e) => {
                    let conn = state.db.lock().unwrap();
                    let _ = db::update_feed_error(&conn, feed_id, true);
                    Err(format!("Parse error: {}", e))
                },
            }
        },
        Err(e) => {
            let conn = state.db.lock().unwrap();
            let _ = db::update_feed_error(&conn, feed_id, true);
            Err(format!("Network error: {}", e))
        },
    }
}

#[tauri::command]
pub async fn refresh_all_feeds(state: State<'_, AppState>) -> Result<usize, String> {
    let feeds = {
        let conn = state.db.lock().unwrap();
        let folders = db::get_folders_with_feeds(&conn).map_err(|e| e.to_string())?;
        folders
            .into_iter()
            .flat_map(|f| f.feeds)
            .collect::<Vec<_>>()
    };

    let mut total = 0;
    for feed in feeds {
        if let Ok(count) = refresh_feed(feed.id, state.clone()).await {
            total += count;
        }
    }
    Ok(total)
}

#[tauri::command]
pub async fn add_feed(
    url: String,
    folder_id: Option<i64>,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let client = create_client()?;
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;

    let original_url = response.url().clone();

    let content_bytes = response.bytes().await.map_err(|e| e.to_string())?;

    // Always attempt direct parse first — some servers (e.g. Squarespace) serve
    // RSS/Atom with text/html content-type, so we cannot rely on headers alone.
    let initial_parse = match feed_rs::parser::parse(Cursor::new(content_bytes.clone())) {
        Ok(f) if f.title.is_some() || !f.entries.is_empty() => Some(f),
        _ => None,
    };

    let (feed, final_url) = if let Some(f) = initial_parse {
        (f, url)
    } else {
        let discovered_url_str = {
            let html_content = String::from_utf8_lossy(&content_bytes);
            let document = Html::parse_document(&html_content);
            // Select all <link> elements and filter in Rust — avoids CSS quoted-attribute
            // parsing inconsistencies in the scraper crate.
            let feed_types = ["application/rss+xml", "application/atom+xml", "application/feed+json"];
            let found = Selector::parse("link")
                .ok()
                .and_then(|sel| {
                    document.select(&sel).find_map(|el| {
                        let t = el.value().attr("type").unwrap_or("");
                        if feed_types.iter().any(|ft| t.contains(ft)) {
                            el.value().attr("href").map(|h| h.to_string())
                        } else {
                            None
                        }
                    })
                });

            found.and_then(|href| {
                Url::parse(original_url.as_str())
                    .and_then(|base| base.join(&href))
                    .ok()
                    .map(|u| u.to_string())
            })
        };

        if let Some(new_url) = discovered_url_str {
            let resp = client
                .get(&new_url)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
            match feed_rs::parser::parse(Cursor::new(bytes)) {
                Ok(f) => (f, new_url),
                Err(e) => return Err(format!("Discovered feed failed to parse: {}", e)),
            }
        } else {
            return Err("No valid feed found".to_string());
        }
    };

    let title = feed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| "Untitled Feed".to_string());

    let id = {
        let conn = state.db.lock().unwrap();
        let target = folder_id.unwrap_or(1);
        db::create_feed(&conn, &title, &final_url, target).map_err(|e| e.to_string())?;
        conn.query_row("SELECT id FROM feeds WHERE url = ?1", [&final_url], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?
    };

    let _ = refresh_feed(id, state).await;

    Ok(id)
}

#[tauri::command]
pub fn rename_folder(id: i64, new_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::rename_folder(&conn, id, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_feed(id: i64, new_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::rename_feed(&conn, id, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_feed(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::delete_feed(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_folder(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::delete_folder(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_feed(feed_id: i64, folder_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::move_feed(&conn, feed_id, folder_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_articles(
    query: String,
    limit: usize,
    offset: usize,
    sort_desc: bool,
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::Article>, String> {
    let conn = state.db.lock().unwrap();
    db::search_articles(&conn, &query, limit, offset, !sort_desc).map_err(|e| e.to_string())
}