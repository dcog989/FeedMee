use crate::{
    AppState, db,
    models::{Article, Folder},
    settings::{self, AppSettings},
};
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use readabilityrs::{Readability, ReadabilityOptions};
use scraper::{Html, Selector};
use serde::Serialize;
use std::fmt::Write;
use std::fs;

#[derive(Serialize)]
pub struct AppInfo {
    pub version: String,
    pub data_path: String,
    pub logs_path: String,
    pub db_path: String,
}

#[tauri::command]
pub fn get_app_info(app: tauri::AppHandle) -> Result<AppInfo, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    let version = app.package_info().version.to_string();

    Ok(AppInfo {
        version,
        data_path: app_data_dir.to_string_lossy().to_string(),
        logs_path: app_data_dir.join("Logs").to_string_lossy().to_string(),
        db_path: app_data_dir
            .join("Database")
            .join("feedmee.sqlite")
            .to_string_lossy()
            .to_string(),
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
pub fn get_shortcuts(
    app: tauri::AppHandle,
) -> Result<std::collections::HashMap<String, String>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let shortcuts_path = app_data_dir.join("shortcuts.json");

    if shortcuts_path.exists() {
        let content = fs::read_to_string(&shortcuts_path).map_err(|e| e.to_string())?;
        let shortcuts: std::collections::HashMap<String, String> =
            serde_json::from_str(&content).unwrap_or_default();
        Ok(shortcuts)
    } else {
        Ok(std::collections::HashMap::new())
    }
}

#[tauri::command]
pub fn save_shortcuts(
    shortcuts: std::collections::HashMap<String, String>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let shortcuts_path = app_data_dir.join("shortcuts.json");

    let json = serde_json::to_string_pretty(&shortcuts).map_err(|e| e.to_string())?;
    fs::write(shortcuts_path, json).map_err(|e| e.to_string())?;

    info!("Shortcuts saved to disk");
    Ok(())
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
                        let _ = db::create_feed(&conn, &child.text, &url, folder_id, "rss");
                    }
                }
            }
        } else if let Some(url) = outline.xml_url {
            let _ = db::create_feed(&conn, &outline.text, &url, default_folder_id, "rss");
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

    let options = ReadabilityOptions::default();
    let readability =
        Readability::new(&html, Some(&url), Some(options)).map_err(|e| format!("{:?}", e))?;
    let article = readability.parse().ok_or("Failed to parse content")?;
    article.content.ok_or("No content extracted".to_string())
}

#[tauri::command]
pub async fn refresh_feed(feed_id: i64, state: State<'_, AppState>) -> Result<usize, String> {
    let (url, feed_type, _stored_hash) = {
        let conn = state.db.lock().unwrap();
        let feed = db::get_feed(&conn, feed_id).map_err(|e| e.to_string())?;
        (feed.url, feed.feed_type, feed.content_hash)
    };

    let client = create_client()?;

    // Check if this is a website feed (or legacy feed without feed_type)
    let is_website = feed_type == "website" || feed_type.is_empty();
    debug!(
        "refresh_feed: feed_id={}, url={}, feed_type='{}', is_website={}",
        feed_id, url, feed_type, is_website
    );

    if is_website {
        let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
        let html = response.text().await.map_err(|e| e.to_string())?;
        let articles = scrape_articles_from_page(&html, &url);
        let conn = state.db.lock().unwrap();
        let count = articles
            .into_iter()
            .filter_map(|a| db::insert_article(&conn, &a).ok().map(|_| 1usize))
            .sum();
        let _ = db::update_feed_error(&conn, feed_id, false);
        return Ok(count);
    }

    // Default: RSS/Atom feed handling
    let result = client.get(&url).send().await;

    match result {
        Ok(response) => {
            let content = response.bytes().await.map_err(|e| e.to_string())?;
            match feed_rs::parser::parse(Cursor::new(content)) {
                Ok(feed) => {
                    info!(
                        "refresh_feed: parsed feed ok, {} entries",
                        feed.entries.len()
                    );
                    let conn = state.db.lock().unwrap();
                    let mut count = 0;
                    for entry in feed.entries {
                        let article_url = entry
                            .links
                            .iter()
                            .find(|l| l.rel.as_deref() == Some("alternate"))
                            .or(entry.links.first())
                            .map(|l| l.href.clone())
                            .unwrap_or_else(|| {
                                // Generate a stable synthetic URL so the UNIQUE constraint can
                                // still deduplicate and the article can be stored.
                                let key = if !entry.id.is_empty() {
                                    entry.id.clone()
                                } else {
                                    entry
                                        .title
                                        .as_ref()
                                        .map(|t| t.content.clone())
                                        .unwrap_or_default()
                                };
                                format!(
                                    "{}/#{}",
                                    url.trim_end_matches('/'),
                                    compute_content_hash(&key)
                                )
                            });

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
                        match db::insert_article(&conn, &article) {
                            Ok(_) => count += 1,
                            Err(e) => error!(
                                "refresh_feed: insert_article failed for url={}: {}",
                                article.url, e
                            ),
                        }
                    }
                    let _ = db::update_feed_error(&conn, feed_id, false);
                    Ok(count)
                },
                Err(e) => {
                    error!("refresh_feed: feed_rs parse error for {}: {}", url, e);
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

fn compute_content_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Scrape a listing page for article links. Returns Article structs with empty
/// summary (content is fetched on-demand via get_article_content). Links are
/// filtered to same-domain, non-trivial hrefs, and deduplicated.
fn scrape_articles_from_page(html: &str, page_url: &str) -> Vec<Article> {
    debug!("scrape_articles_from_page: url={}", page_url);
    let base = match Url::parse(page_url) {
        Ok(u) => u,
        Err(_) => return vec![],
    };
    let base_host = base.host_str().unwrap_or("").to_string();

    let document = Html::parse_document(html);
    let anchor_sel = match Selector::parse("a[href]") {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let now = chrono::Utc::now().timestamp();
    let mut seen = std::collections::HashSet::new();
    let mut articles = Vec::new();

    for el in document.select(&anchor_sel) {
        let href = match el.value().attr("href") {
            Some(h) => h,
            None => continue,
        };

        let abs = match base.join(href) {
            Ok(u) => u,
            Err(_) => continue,
        };

        // Same domain only; skip fragment-only or javascript links
        if abs.host_str().unwrap_or("") != base_host {
            continue;
        }
        if abs.path() == base.path() {
            continue;
        }

        let url_str = abs.to_string();
        if !seen.insert(url_str.clone()) {
            continue;
        }

        // Extract link text as title; fall back to title attr, then URL slug
        let anchor_text: String = el.text().collect::<Vec<_>>().join(" ");
        let anchor_text = anchor_text.split_whitespace().collect::<Vec<_>>().join(" ");

        let title = if anchor_text.len() >= 10 {
            anchor_text.clone()
        } else if let Some(t) = el.value().attr("title").filter(|t| t.len() >= 10) {
            t.to_string()
        } else {
            // Derive title from the URL path slug (last meaningful segment)
            let slug = abs
                .path_segments()
                .and_then(|mut segs| segs.rfind(|s| !s.is_empty() && s.len() > 3))
                .unwrap_or("");
            let from_slug = slug.replace(['-', '_'], " ");
            if from_slug.len() >= 10 {
                // Title-case the slug
                from_slug
                    .split_whitespace()
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                debug!("scrape: skipping, no usable title for {}", url_str);
                continue;
            }
        };

        // Skip navigation/category URLs - only keep URLs that look like articles
        // (have a path depth of at least 2 segments, or passed the anchor text check)
        let path_depth = abs
            .path_segments()
            .map(|s| s.filter(|p| !p.is_empty()).count())
            .unwrap_or(0);
        if path_depth < 2 && anchor_text.len() < 10 {
            debug!("scrape: skipping shallow nav url {}", url_str);
            continue;
        }

        debug!("scrape: accepting {:?} -> {}", title, url_str);
        articles.push(Article {
            id: 0,
            feed_id: 0, // caller sets this
            title,
            author: String::new(),
            summary: String::new(),
            url: url_str,
            timestamp: now,
            is_read: false,
            is_saved: false,
        });
    }

    articles
}

async fn add_website_feed(
    url: &str,
    content_bytes: &[u8],
    folder_id: Option<i64>,
    state: &State<'_, AppState>,
) -> Result<i64, String> {
    // Extract page title from <title> tag for the feed name
    let html = String::from_utf8_lossy(content_bytes);
    let document = Html::parse_document(&html);
    let title_sel = Selector::parse("title").ok();
    let title = title_sel
        .and_then(|sel| document.select(&sel).next())
        .map(|el| el.text().collect::<String>())
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| url.to_string());

    let feed_id = {
        let conn = state.db.lock().unwrap();
        let target = folder_id.unwrap_or(1);
        db::create_feed(&conn, &title, url, target, "website").map_err(|e| e.to_string())?;
        conn.query_row("SELECT id FROM feeds WHERE url = ?1", [url], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?
    };

    let mut articles = scrape_articles_from_page(&html, url);
    for a in &mut articles {
        a.feed_id = feed_id;
    }

    if articles.is_empty() {
        return Err(format!("No articles found on page: {}", url));
    }

    let conn = state.db.lock().unwrap();
    for article in articles {
        let _ = db::insert_article(&conn, &article);
    }

    Ok(feed_id)
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

    // Try direct RSS parse first
    let initial_parse = match feed_rs::parser::parse(Cursor::new(content_bytes.clone())) {
        Ok(f) if !f.entries.is_empty() => Some((f, url.clone())),
        _ => None,
    };

    debug!(
        "add_feed: url={}, initial_parse={}",
        url,
        initial_parse.is_some()
    );

    let (feed, final_url, feed_type) = if let Some((f, u)) = initial_parse {
        (f, u, "rss".to_string())
    } else {
        // Try to discover RSS in HTML
        let discovered_url_str = {
            let html_content = String::from_utf8_lossy(&content_bytes);
            debug!(
                "add_feed: HTML preview (first 1000): {}",
                &html_content[..html_content.len().min(1000)]
            );
            let document = Html::parse_document(&html_content);
            let feed_types = [
                "application/rss+xml",
                "application/atom+xml",
                "application/feed+json",
            ];
            let found = Selector::parse("link").ok().and_then(|sel| {
                let all_links: Vec<_> = document.select(&sel).collect();
                debug!("add_feed: found {} <link> tags", all_links.len());
                for el in &all_links {
                    let t = el.value().attr("type").unwrap_or("");
                    let h = el.value().attr("href").unwrap_or("");
                    if !t.is_empty() {
                        debug!("add_feed: <link type={:?} href={:?}>", t, h);
                    }
                }
                all_links.into_iter().find_map(|el| {
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
            debug!("add_feed: discovered RSS url={}", new_url);
            let resp = client
                .get(&new_url)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
            match feed_rs::parser::parse(Cursor::new(bytes.clone())) {
                Ok(f) => {
                    info!(
                        "add_feed: RSS parse ok, {} entries, title={:?}",
                        f.entries.len(),
                        f.title.as_ref().map(|t| &t.content)
                    );
                    if f.entries.is_empty() {
                        info!(
                            "add_feed: RSS feed is empty, falling back to website scraping for {}",
                            url
                        );
                        return add_website_feed(&url, &content_bytes, folder_id, &state).await;
                    }
                    (f, new_url, "rss".to_string())
                },
                Err(e) => {
                    error!("add_feed: RSS parse failed for {}: {}", new_url, e);
                    // Log first 500 bytes of response for diagnosis
                    let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(500)]);
                    error!("add_feed: response preview: {}", preview);
                    return add_website_feed(&url, &content_bytes, folder_id, &state).await;
                },
            }
        } else {
            debug!("add_feed: no RSS found, treating as website");
            return add_website_feed(&url, &content_bytes, folder_id, &state).await;
        }
    };

    let title = feed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| "Untitled Feed".to_string());

    let id = {
        let conn = state.db.lock().unwrap();
        let target = folder_id.unwrap_or(1);
        db::create_feed(&conn, &title, &final_url, target, &feed_type)
            .map_err(|e| e.to_string())?;
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
