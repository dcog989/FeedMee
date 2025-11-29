use crate::{
    AppState, db,
    models::{Article, Folder},
};
use std::io::Cursor;
use tauri::State;

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
pub async fn import_opml(path: String, state: State<'_, AppState>) -> Result<(), String> {
    // 1. Read file content
    let xml_content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

    // 2. Parse OPML
    let document = opml::OPML::from_str(&xml_content).map_err(|e| e.to_string())?;

    // 3. Insert into DB
    let conn = state.db.lock().unwrap();

    // Default folder for loose feeds
    let default_folder_id = db::create_folder(&conn, "Uncategorized").map_err(|e| e.to_string())?;

    for outline in document.body.outlines {
        // Case A: It's a Folder (contains other outlines)
        if !outline.outlines.is_empty() {
            let folder_name = outline.text;
            let folder_id = db::create_folder(&conn, &folder_name).map_err(|e| e.to_string())?;

            for child in outline.outlines {
                if let Some(url) = child.xml_url {
                    let _ = db::create_feed(&conn, &child.text, &url, folder_id);
                }
            }
        }
        // Case B: It's a top-level Feed
        else if let Some(url) = outline.xml_url {
            let _ = db::create_feed(&conn, &outline.text, &url, default_folder_id);
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn refresh_feed(feed_id: i64, state: State<'_, AppState>) -> Result<usize, String> {
    // 1. Get URL from DB
    let url = {
        let conn = state.db.lock().unwrap();
        db::get_feed_url(&conn, feed_id).map_err(|e| e.to_string())?
    };

    // 2. Fetch Feed (HTTP)
    let content = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch feed: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read bytes: {}", e))?;

    // 3. Parse Feed
    let feed = feed_rs::parser::parse(Cursor::new(content))
        .map_err(|e| format!("Failed to parse feed: {}", e))?;

    // 4. Save Articles to DB
    let conn = state.db.lock().unwrap();
    let mut count = 0;

    for entry in feed.entries {
        let article = Article {
            id: 0, // DB handles auto-increment
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
    // 1. Fetch the feed to validate and get title
    let content = reqwest::get(&url)
        .await
        .map_err(|e| format!("Network error: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Read error: {}", e))?;

    let feed =
        feed_rs::parser::parse(Cursor::new(content)).map_err(|e| format!("Parse error: {}", e))?;

    let title = feed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| "Untitled Feed".to_string());

    // 2. Insert into DB
    let conn = state.db.lock().unwrap();

    // Use provided folder or default to ID 1 (Tech News) or create "Inbox"
    let target_folder = folder_id.unwrap_or(1);

    // We need to update db.rs to return the ID of the created feed
    db::create_feed(&conn, &title, &url, target_folder).map_err(|e| e.to_string())?;

    // Retrieve the ID (hacky since create_feed doesn't return it yet, but sufficient for now)
    let id: i64 = conn
        .query_row("SELECT id FROM feeds WHERE url = ?1", [&url], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;

    Ok(id)
}
