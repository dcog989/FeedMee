use crate::{
    AppState, db,
    models::{Article, Folder},
};
use log::{debug, error, info, warn};
use readability_rust::Readability;
use scraper::{Html, Selector};
use std::fmt::Write;
use std::io::Cursor;
use tauri::State;
use url::Url;

#[tauri::command]
pub fn get_folders_with_feeds(state: State<'_, AppState>) -> Result<Vec<Folder>, String> {
    debug!("Getting folders with feeds");
    let conn = state.db.lock().unwrap();
    match db::get_folders_with_feeds(&conn) {
        Ok(folders) => {
            info!("Retrieved {} folders with feeds", folders.len());
            Ok(folders)
        }
        Err(e) => {
            error!("Failed to get folders with feeds: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn get_articles_for_feed(
    feed_id: i64,
    limit: usize,
    offset: usize,
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    debug!(
        "Getting articles for feed_id={}, limit={}, offset={}",
        feed_id, limit, offset
    );
    let conn = state.db.lock().unwrap();
    match db::get_articles_for_feed(&conn, feed_id, limit, offset) {
        Ok(articles) => {
            info!(
                "Retrieved {} articles for feed_id={}",
                articles.len(),
                feed_id
            );
            Ok(articles)
        }
        Err(e) => {
            error!("Failed to get articles for feed_id={}: {}", feed_id, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn get_latest_articles(
    cutoff_timestamp: i64,
    limit: usize,
    offset: usize,
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    debug!(
        "Getting latest articles: cutoff={}, limit={}, offset={}",
        cutoff_timestamp, limit, offset
    );
    let conn = state.db.lock().unwrap();
    match db::get_latest_articles(&conn, cutoff_timestamp, limit, offset) {
        Ok(articles) => {
            info!("Retrieved {} latest articles", articles.len());
            Ok(articles)
        }
        Err(e) => {
            error!("Failed to get latest articles: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn get_saved_articles(
    limit: usize,
    offset: usize,
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    debug!("Getting saved articles: limit={}, offset={}", limit, offset);
    let conn = state.db.lock().unwrap();
    match db::get_saved_articles(&conn, limit, offset) {
        Ok(articles) => {
            info!("Retrieved {} saved articles", articles.len());
            Ok(articles)
        }
        Err(e) => {
            error!("Failed to get saved articles: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn create_folder(name: String, state: State<'_, AppState>) -> Result<i64, String> {
    info!("Creating folder: {}", name);
    let conn = state.db.lock().unwrap();
    match db::create_folder(&conn, &name) {
        Ok(id) => {
            info!("Created folder '{}' with id={}", name, id);
            Ok(id)
        }
        Err(e) => {
            error!("Failed to create folder '{}': {}", name, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn mark_article_saved(
    id: i64,
    is_saved: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    debug!("Marking article id={} as saved={}", id, is_saved);
    let conn = state.db.lock().unwrap();
    match db::update_article_saved(&conn, id, is_saved) {
        Ok(_) => {
            info!("Article id={} saved status updated to {}", id, is_saved);
            Ok(())
        }
        Err(e) => {
            error!("Failed to update saved status for article id={}: {}", id, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn import_opml(path: String, state: State<'_, AppState>) -> Result<(), String> {
    info!("Importing OPML from: {}", path);

    let xml_content = std::fs::read_to_string(&path).map_err(|e| {
        error!("Failed to read OPML file '{}': {}", path, e);
        e.to_string()
    })?;

    let document = opml::OPML::from_str(&xml_content).map_err(|e| {
        error!("Failed to parse OPML file '{}': {}", path, e);
        e.to_string()
    })?;

    let conn = state.db.lock().unwrap();
    let default_folder_id = db::create_folder(&conn, "Uncategorized").map_err(|e| {
        error!("Failed to create default folder: {}", e);
        e.to_string()
    })?;

    let mut folder_count = 0;
    let mut feed_count = 0;

    for outline in document.body.outlines {
        if !outline.outlines.is_empty() {
            // This is a folder with feeds
            let folder_name = outline.text;
            match db::create_folder(&conn, &folder_name) {
                Ok(folder_id) => {
                    debug!("Created folder: {}", folder_name);
                    folder_count += 1;

                    for child in outline.outlines {
                        if let Some(url) = child.xml_url {
                            match db::create_feed(&conn, &child.text, &url, folder_id) {
                                Ok(_) => {
                                    debug!(
                                        "Added feed '{}' to folder '{}'",
                                        child.text, folder_name
                                    );
                                    feed_count += 1;
                                }
                                Err(e) => {
                                    warn!("Failed to add feed '{}': {}", child.text, e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to create folder '{}': {}", folder_name, e);
                    return Err(e.to_string());
                }
            }
        } else if let Some(url) = outline.xml_url {
            // This is a feed without a folder
            match db::create_feed(&conn, &outline.text, &url, default_folder_id) {
                Ok(_) => {
                    debug!("Added feed '{}' to default folder", outline.text);
                    feed_count += 1;
                }
                Err(e) => {
                    warn!("Failed to add feed '{}': {}", outline.text, e);
                }
            }
        }
    }

    info!(
        "OPML import complete: {} folders, {} feeds imported",
        folder_count, feed_count
    );
    Ok(())
}

#[tauri::command]
pub async fn export_opml(state: State<'_, AppState>) -> Result<String, String> {
    info!("Exporting OPML");

    let folders = {
        let conn = state.db.lock().unwrap();
        db::get_folders_with_feeds(&conn).map_err(|e| {
            error!("Failed to get folders for export: {}", e);
            e.to_string()
        })?
    };

    let mut opml = String::new();
    writeln!(&mut opml, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>").unwrap();
    writeln!(&mut opml, "<opml version=\"2.0\">").unwrap();
    writeln!(&mut opml, "  <head><title>FeedMee Export</title></head>").unwrap();
    writeln!(&mut opml, "  <body>").unwrap();

    let mut total_feeds = 0;
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
            total_feeds += 1;
        }
        writeln!(&mut opml, "    </outline>").unwrap();
    }
    writeln!(&mut opml, "  </body>").unwrap();
    writeln!(&mut opml, "</opml>").unwrap();

    info!("OPML export complete: {} feeds exported", total_feeds);
    Ok(opml)
}

#[tauri::command]
pub async fn write_file(path: String, content: String) -> Result<(), String> {
    info!("Writing file to: {}", path);
    std::fs::write(&path, content).map_err(|e| {
        error!("Failed to write file '{}': {}", path, e);
        e.to_string()
    })?;
    info!("File written successfully: {}", path);
    Ok(())
}

#[tauri::command]
pub async fn get_article_content(url: String) -> Result<String, String> {
    info!("Fetching full article content from: {}", url);

    let html = reqwest::get(&url)
        .await
        .map_err(|e| {
            error!("Failed to fetch URL '{}': {}", url, e);
            format!("Failed to fetch URL: {}", e)
        })?
        .text()
        .await
        .map_err(|e| {
            error!("Failed to read text from '{}': {}", url, e);
            format!("Failed to read text: {}", e)
        })?;

    debug!("Parsing article content with Readability");
    let mut parser = Readability::new(&html, None).map_err(|e| {
        error!("Readability init error for '{}': {:?}", url, e);
        format!("Readability init error: {:?}", e)
    })?;

    let article = parser.parse().ok_or_else(|| {
        warn!("Failed to parse readability content from '{}'", url);
        "Failed to parse readability content".to_string()
    })?;

    let content = article.content.ok_or_else(|| {
        warn!("No content extracted from '{}'", url);
        "No content extracted".to_string()
    })?;

    info!("Successfully extracted article content from: {}", url);
    Ok(content)
}

#[tauri::command]
pub fn get_articles_for_folder(
    folder_id: i64,
    limit: usize,
    offset: usize,
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    debug!(
        "Getting articles for folder_id={}, limit={}, offset={}",
        folder_id, limit, offset
    );
    let conn = state.db.lock().unwrap();
    match db::get_articles_for_folder(&conn, folder_id, limit, offset) {
        Ok(articles) => {
            info!(
                "Retrieved {} articles for folder_id={}",
                articles.len(),
                folder_id
            );
            Ok(articles)
        }
        Err(e) => {
            error!("Failed to get articles for folder_id={}: {}", folder_id, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn refresh_feed(feed_id: i64, state: State<'_, AppState>) -> Result<usize, String> {
    info!("Refreshing feed_id={}", feed_id);

    let url = {
        let conn = state.db.lock().unwrap();
        db::get_feed_url(&conn, feed_id).map_err(|e| {
            error!("Failed to get URL for feed_id={}: {}", feed_id, e);
            e.to_string()
        })?
    };

    debug!("Fetching feed from: {}", url);
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    let content = client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to fetch feed from '{}': {}", url, e);
            format!("Failed to fetch feed: {}", e)
        })?
        .bytes()
        .await
        .map_err(|e| {
            error!("Failed to read bytes from '{}': {}", url, e);
            format!("Failed to read bytes: {}", e)
        })?;

    debug!("Parsing feed content");
    let feed = feed_rs::parser::parse(Cursor::new(content)).map_err(|e| {
        error!("Failed to parse feed from '{}': {}", url, e);
        format!("Failed to parse feed: {}", e)
    })?;

    let entry_count = feed.entries.len();
    debug!("Feed contains {} entries", entry_count);

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

    info!(
        "Feed refresh complete for feed_id={}: {} new articles added out of {} entries",
        feed_id, count, entry_count
    );
    Ok(count)
}

#[tauri::command]
pub async fn refresh_all_feeds(state: State<'_, AppState>) -> Result<usize, String> {
    info!("Refreshing all feeds");
    let feeds = {
        let conn = state.db.lock().unwrap();
        // Flatten folders to get all feeds
        let folders = db::get_folders_with_feeds(&conn).map_err(|e| e.to_string())?;
        let mut all_feeds = Vec::new();
        for folder in folders {
            for feed in folder.feeds {
                all_feeds.push(feed);
            }
        }
        all_feeds
    };

    let mut total_new = 0;
    // Note: In a production app, we might want to do this concurrently (e.g. JoinSet)
    // For now, sequential is safer for SQLite locking.
    for feed in feeds {
        match refresh_feed(feed.id, state.clone()).await {
            Ok(count) => total_new += count,
            Err(e) => warn!("Failed to refresh feed {}: {}", feed.name, e),
        }
    }

    Ok(total_new)
}

#[tauri::command]
pub async fn add_feed(
    url: String,
    folder_id: Option<i64>,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    info!("Adding new feed from URL: {}", url);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(&url).send().await.map_err(|e| {
        error!("Network error while fetching '{}': {}", url, e);
        format!("Network error: {}", e)
    })?;

    let original_url = response.url().clone();
    debug!("Original URL after redirects: {}", original_url);

    let content_bytes = response.bytes().await.map_err(|e| {
        error!("Read error for '{}': {}", url, e);
        format!("Read error: {}", e)
    })?;

    let feed_parse_result = feed_rs::parser::parse(Cursor::new(content_bytes.clone()));

    let (final_url, feed) = match feed_parse_result {
        Ok(f) => {
            debug!("Successfully parsed feed directly from URL");
            (url, f)
        }
        Err(parse_err) => {
            warn!(
                "Failed to parse feed directly, attempting feed discovery: {}",
                parse_err
            );

            let discovered_url = {
                let html_content = String::from_utf8_lossy(&content_bytes);
                let document = Html::parse_document(&html_content);
                let selector = Selector::parse(
                    "link[type='application/rss+xml'], link[type='application/atom+xml']",
                )
                .map_err(|_| {
                    error!("Internal selector parse error");
                    "Internal error"
                })?;

                if let Some(element) = document.select(&selector).next() {
                    if let Some(href) = element.value().attr("href") {
                        Url::parse(original_url.as_str())
                            .and_then(|base| base.join(href))
                            .map(|u| {
                                debug!("Discovered feed URL: {}", u);
                                u.to_string()
                            })
                            .ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            if let Some(feed_url) = discovered_url {
                debug!("Fetching discovered feed: {}", feed_url);
                let discovered_content = client
                    .get(&feed_url)
                    .send()
                    .await
                    .map_err(|e| {
                        error!("Fetch discovered feed error for '{}': {}", feed_url, e);
                        format!("Fetch discovered error: {}", e)
                    })?
                    .bytes()
                    .await
                    .map_err(|e| {
                        error!("Read discovered feed error for '{}': {}", feed_url, e);
                        format!("Read discovered error: {}", e)
                    })?;

                let f = feed_rs::parser::parse(Cursor::new(discovered_content)).map_err(|e| {
                    error!("Discovered feed parse error for '{}': {}", feed_url, e);
                    format!("Discovered parse error: {}", e)
                })?;

                info!("Successfully parsed discovered feed");
                (feed_url, f)
            } else {
                error!("No RSS/Atom feed found at '{}'", url);
                return Err("No RSS/Atom feed found".into());
            }
        }
    };

    let title = feed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| "Untitled Feed".to_string());

    debug!("Feed title: {}", title);

    let conn = state.db.lock().unwrap();
    let target_folder = folder_id.unwrap_or(1);

    debug!("Creating feed in folder_id={}", target_folder);
    let _ = db::create_feed(&conn, &title, &final_url, target_folder);

    let id: i64 = conn
        .query_row("SELECT id FROM feeds WHERE url = ?1", [&final_url], |row| {
            row.get(0)
        })
        .map_err(|e| {
            error!("Failed to retrieve feed id for '{}': {}", final_url, e);
            e.to_string()
        })?;

    info!(
        "Feed '{}' added successfully with id={} in folder_id={}",
        title, id, target_folder
    );
    Ok(id)
}

#[tauri::command]
pub fn rename_folder(id: i64, new_name: String, state: State<'_, AppState>) -> Result<(), String> {
    info!("Renaming folder id={} to '{}'", id, new_name);
    let conn = state.db.lock().unwrap();
    match db::rename_folder(&conn, id, &new_name) {
        Ok(_) => {
            info!("Folder id={} renamed to '{}'", id, new_name);
            Ok(())
        }
        Err(e) => {
            error!("Failed to rename folder id={}: {}", id, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn delete_feed(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    info!("Deleting feed id={}", id);
    let conn = state.db.lock().unwrap();
    match db::delete_feed(&conn, id) {
        Ok(_) => {
            info!("Feed id={} deleted successfully", id);
            Ok(())
        }
        Err(e) => {
            error!("Failed to delete feed id={}: {}", id, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn delete_folder(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    info!("Deleting folder id={}", id);
    let conn = state.db.lock().unwrap();
    match db::delete_folder(&conn, id) {
        Ok(_) => {
            info!("Folder id={} deleted successfully", id);
            Ok(())
        }
        Err(e) => {
            error!("Failed to delete folder id={}: {}", id, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn move_feed(feed_id: i64, folder_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    info!("Moving feed_id={} to folder_id={}", feed_id, folder_id);
    let conn = state.db.lock().unwrap();
    match db::move_feed(&conn, feed_id, folder_id) {
        Ok(_) => {
            info!(
                "Feed feed_id={} moved to folder_id={} successfully",
                feed_id, folder_id
            );
            Ok(())
        }
        Err(e) => {
            error!(
                "Failed to move feed_id={} to folder_id={}: {}",
                feed_id, folder_id, e
            );
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn mark_article_read(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    // No logging here to reduce noise for frequent actions, or use debug!
    debug!("Marking article id={} as read", id);
    let conn = state.db.lock().unwrap();
    match db::mark_article_read(&conn, id) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to mark article id={} as read: {}", id, e);
            Err(e.to_string())
        }
    }
}
