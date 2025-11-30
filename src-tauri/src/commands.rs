use crate::{
    AppState, db,
    models::{Article, Folder},
};
use readability_rust::Readability;
use scraper::{Html, Selector};
use std::fmt::Write;
use std::io::Cursor;
use tauri::State;
use url::Url;

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
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    let conn = state.db.lock().unwrap();
    db::get_articles_for_feed(&conn, feed_id, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_latest_articles(
    cutoff_timestamp: i64,
    limit: usize,
    offset: usize,
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    let conn = state.db.lock().unwrap();
    db::get_latest_articles(&conn, cutoff_timestamp, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_saved_articles(
    limit: usize,
    offset: usize,
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    let conn = state.db.lock().unwrap();
    db::get_saved_articles(&conn, limit, offset).map_err(|e| e.to_string())
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
pub async fn import_opml(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let xml_content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let document = opml::OPML::from_str(&xml_content).map_err(|e| e.to_string())?;
    let conn = state.db.lock().unwrap();
    let default_folder_id = db::create_folder(&conn, "Uncategorized").map_err(|e| e.to_string())?;

    for outline in document.body.outlines {
        if !outline.outlines.is_empty() {
            let folder_name = outline.text;
            let folder_id = db::create_folder(&conn, &folder_name).map_err(|e| e.to_string())?;
            for child in outline.outlines {
                if let Some(url) = child.xml_url {
                    let _ = db::create_feed(&conn, &child.text, &url, folder_id);
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
        for feed in folder.feeds {
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
    std::fs::write(path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_article_content(url: String) -> Result<String, String> {
    let html = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch URL: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read text: {}", e))?;

    let mut parser =
        Readability::new(&html, None).map_err(|e| format!("Readability init error: {:?}", e))?;

    let article = parser
        .parse()
        .ok_or("Failed to parse readability content".to_string())?;

    article.content.ok_or("No content extracted".to_string())
}

#[tauri::command]
pub async fn refresh_feed(feed_id: i64, state: State<'_, AppState>) -> Result<usize, String> {
    let url = {
        let conn = state.db.lock().unwrap();
        db::get_feed_url(&conn, feed_id).map_err(|e| e.to_string())?
    };

    let content = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch feed: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read bytes: {}", e))?;

    let feed = feed_rs::parser::parse(Cursor::new(content))
        .map_err(|e| format!("Failed to parse feed: {}", e))?;

    let conn = state.db.lock().unwrap();
    let mut count = 0;

    for entry in feed.entries {
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
            url: entry
                .links
                .first()
                .map(|l| l.href.clone())
                .unwrap_or_default(),
            timestamp: entry
                .published
                .or(entry.updated)
                .map(|d| d.timestamp())
                .unwrap_or(0),
            is_read: false,
            is_saved: false,
        };

        if !article.url.is_empty() {
            if let Ok(_) = db::insert_article(&conn, &article) {
                count += 1;
            }
        }
    }
    Ok(count)
}

#[tauri::command]
pub async fn add_feed(
    url: String,
    folder_id: Option<i64>,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    let original_url = response.url().clone();
    let content_bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Read error: {}", e))?;
    let feed_parse_result = feed_rs::parser::parse(Cursor::new(content_bytes.clone()));

    let (final_url, feed) = match feed_parse_result {
        Ok(f) => (url, f),
        Err(_) => {
            let discovered_url = {
                let html_content = String::from_utf8_lossy(&content_bytes);
                let document = Html::parse_document(&html_content);
                let selector = Selector::parse(
                    "link[type='application/rss+xml'], link[type='application/atom+xml']",
                )
                .map_err(|_| "Internal error")?;
                if let Some(element) = document.select(&selector).next() {
                    if let Some(href) = element.value().attr("href") {
                        Url::parse(original_url.as_str())
                            .and_then(|base| base.join(href))
                            .map(|u| u.to_string())
                            .ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            if let Some(feed_url) = discovered_url {
                let discovered_content = reqwest::get(&feed_url)
                    .await
                    .map_err(|e| format!("Fetch discovered error: {}", e))?
                    .bytes()
                    .await
                    .map_err(|e| format!("Read discovered error: {}", e))?;
                let f = feed_rs::parser::parse(Cursor::new(discovered_content))
                    .map_err(|e| format!("Discovered parse error: {}", e))?;
                (feed_url, f)
            } else {
                return Err("No RSS/Atom feed found".into());
            }
        }
    };

    let title = feed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| "Untitled Feed".to_string());
    let conn = state.db.lock().unwrap();
    let target_folder = folder_id.unwrap_or(1);

    // Attempt creation
    let _ = db::create_feed(&conn, &title, &final_url, target_folder);

    // Retrieve ID
    let id: i64 = conn
        .query_row("SELECT id FROM feeds WHERE url = ?1", [&final_url], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub fn rename_folder(id: i64, new_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::rename_folder(&conn, id, &new_name).map_err(|e| e.to_string())
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
