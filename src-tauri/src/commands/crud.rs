use crate::{
    AppState, db,
    models::{Article, Folder, Tag},
};
use log::info;
use rusqlite::Connection;
use tauri::{Manager, State};

#[tauri::command]
pub fn get_folders_with_feeds(state: State<'_, AppState>) -> Result<Vec<Folder>, String> {
    let conn = state.db.lock().unwrap();
    db::get_folders_with_feeds(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_feed_unread_count(feed_id: i64, state: State<'_, AppState>) -> Result<i64, String> {
    let conn = state.db.lock().unwrap();
    db::get_feed_unread_count(&conn, feed_id).map_err(|e| e.to_string())
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
    db::get_articles_for_feed(&conn, feed_id, limit, offset, sort_desc).map_err(|e| e.to_string())
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
    db::get_articles_for_folder(&conn, folder_id, limit, offset, sort_desc).map_err(|e| e.to_string())
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
    db::get_latest_articles(&conn, cutoff_timestamp, limit, offset, sort_desc).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_saved_articles(
    limit: usize,
    offset: usize,
    sort_desc: bool,
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    let conn = state.db.lock().unwrap();
    db::get_saved_articles(&conn, limit, offset, sort_desc).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_folder(name: String, state: State<'_, AppState>) -> Result<i64, String> {
    let conn = state.db.lock().unwrap();
    db::create_folder(&conn, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_article_saved(id: i64, is_saved: bool, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::update_article_saved(&conn, id, is_saved).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_article_read(id: i64, read: bool, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::set_article_read(&conn, id, read).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_all_read(target_type: String, id: i64, state: State<'_, AppState>) -> Result<(), String> {
    info!("Mark All Read: type={}, id={}", target_type, id);
    let conn = state.db.lock().unwrap();
    if target_type == "feed" {
        db::mark_feed_read(&conn, id).map_err(|e| e.to_string())
    } else if target_type == "folder" {
        db::mark_folder_read(&conn, id).map_err(|e| e.to_string())
    } else if target_type == "global" {
        db::mark_global_read(&conn).map_err(|e| e.to_string())
    } else {
        Err("Invalid type".to_string())
    }
}

#[tauri::command]
pub async fn import_opml(path: String, app: tauri::AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        import_opml_blocking(&path, &state)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Insert an OPML feed unless the same URL is already subscribed, including
/// another outline earlier in the same file.
fn insert_opml_feed(
    conn: &Connection,
    name: &str,
    url: &str,
    folder_id: Option<i64>,
    declared_type: Option<&str>,
) -> Result<(), String> {
    let Some((url, feed_type)) = normalize_opml_feed(url, declared_type) else {
        return Ok(());
    };
    if db::feed_exists_by_url(conn, &url, None).map_err(|e| e.to_string())? {
        return Ok(());
    }
    db::create_feed(conn, name, &url, folder_id, &feed_type).map_err(|e| e.to_string())?;
    Ok(())
}

/// Map an outline URL to the URL and source type FeedMee stores. Bluesky URLs
/// and the `feed://` spelling identify themselves; the outline's declared type
/// only separates website feeds from plain RSS.
fn normalize_opml_feed(url: &str, declared_type: Option<&str>) -> Option<(String, String)> {
    if let Some(handle) = url.strip_prefix("bsky:")
        && !handle.is_empty()
    {
        return Some((format!("bsky:{}", handle), "bluesky".to_string()));
    }
    if let Some(rest) = url.strip_prefix("feed://") {
        return Some((format!("https://{}", rest), "rss".to_string()));
    }
    let is_http = url.starts_with("http://") || url.starts_with("https://");
    if !is_http {
        return None;
    }
    let feed_type = if declared_type == Some("website") {
        "website"
    } else {
        "rss"
    };
    Some((url.to_string(), feed_type.to_string()))
}

fn insert_opml_outline(conn: &Connection, outline: &opml::Outline, folder_id: Option<i64>) -> Result<(), String> {
    if let Some(url) = &outline.xml_url {
        return insert_opml_feed(conn, &outline.text, url, folder_id, outline.r#type.as_deref());
    }
    // The app has one folder level, so nested folders collapse into their parent.
    let folder_id = match folder_id {
        Some(id) => id,
        None => db::create_folder(conn, &outline.text).map_err(|e| e.to_string())?,
    };
    for child in &outline.outlines {
        insert_opml_outline(conn, child, Some(folder_id))?;
    }
    Ok(())
}

fn import_opml_blocking(path: &str, state: &AppState) -> Result<(), String> {
    let xml_content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let document = opml::OPML::from_str(&xml_content).map_err(|e| e.to_string())?;
    let conn = state.db.lock().unwrap();
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    for outline in &document.body.outlines {
        insert_opml_outline(&tx, outline, None)?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn export_opml(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let folders = {
        let conn = state.db.lock().unwrap();
        db::get_folders_with_feeds(&conn).map_err(|e| e.to_string())?
    };

    let opml = crate::opml::render(&folders)?;
    std::fs::write(&path, opml).map_err(|e| format!("Failed to write {}: {}", path, e))
}

#[tauri::command]
pub fn rename_folder(id: i64, new_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::rename_folder(&conn, id, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_feed(id: i64, new_name: String, new_url: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    let feed = db::get_feed(&conn, id).map_err(|e| e.to_string())?;
    let url_ok = if feed.feed_type == "bluesky" {
        new_url.starts_with("bsky:")
    } else {
        crate::net::is_public_url(&new_url)
    };
    if !url_ok {
        return Err("Invalid feed URL".to_string());
    }
    if db::feed_exists_by_url(&conn, &new_url, Some(id)).map_err(|e| e.to_string())? {
        return Err("Feed already exists".to_string());
    }
    db::rename_feed(&conn, id, &new_name, &new_url).map_err(|e| e.to_string())
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
pub fn move_feed(feed_id: i64, folder_id: Option<i64>, state: State<'_, AppState>) -> Result<(), String> {
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
    db::search_articles(&conn, &query, limit, offset, sort_desc).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_tags_for_article(article_id: i64, state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let conn = state.db.lock().unwrap();
    db::get_tags_for_article(&conn, article_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let conn = state.db.lock().unwrap();
    db::get_all_tags(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_tag(article_id: i64, name: String, color: String, state: State<'_, AppState>) -> Result<Tag, String> {
    let conn = state.db.lock().unwrap();
    db::add_tag_to_article(&conn, article_id, &name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_tag(article_id: i64, tag_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::remove_tag_from_article(&conn, article_id, tag_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tag(tag_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::delete_tag(&conn, tag_id).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{insert_opml_feed, insert_opml_outline};
    use crate::db;
    use rusqlite::Connection;

    #[test]
    fn opml_import_skips_a_feed_url_that_already_exists() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::init_db(&mut conn).unwrap();

        insert_opml_feed(&conn, "One", "https://example.com/feed", None, None).unwrap();
        insert_opml_feed(&conn, "Two", "https://example.com/feed", None, None).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM feeds WHERE url = ?1",
                ["https://example.com/feed"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn opml_import_rewrites_feed_scheme() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::init_db(&mut conn).unwrap();

        insert_opml_feed(&conn, "One", "feed://example.com/rss", None, None).unwrap();

        let url: String = conn
            .query_row("SELECT url FROM feeds WHERE name = 'One'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(url, "https://example.com/rss");
    }

    #[test]
    fn opml_import_skips_non_http_urls() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::init_db(&mut conn).unwrap();

        insert_opml_feed(&conn, "Bad", "ftp://example.com/rss", None, None).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM feeds WHERE name = 'Bad'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn opml_import_keeps_non_rss_types() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::init_db(&mut conn).unwrap();

        insert_opml_feed(&conn, "Bsky", "bsky:did:plc:abc", None, Some("bluesky")).unwrap();
        insert_opml_feed(&conn, "Site", "https://example.com/", None, Some("website")).unwrap();

        let feed_type = |name: &str| -> String {
            conn.query_row("SELECT feed_type FROM feeds WHERE name = ?1", [name], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(feed_type("Bsky"), "bluesky");
        assert_eq!(feed_type("Site"), "website");
    }

    #[test]
    fn opml_import_flattens_nested_folders() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::init_db(&mut conn).unwrap();

        let folder = opml::Outline {
            text: "Tech".to_string(),
            outlines: vec![opml::Outline {
                text: "AI".to_string(),
                outlines: vec![opml::Outline {
                    text: "AI feed".to_string(),
                    xml_url: Some("https://example.com/ai".to_string()),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        insert_opml_outline(&conn, &folder, None).unwrap();

        let (folder_name, feed_name): (String, String) = conn
            .query_row(
                "SELECT folders.name, feeds.name FROM feeds JOIN folders ON feeds.folder_id = folders.id \
                 WHERE feeds.url = 'https://example.com/ai'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(folder_name, "Tech");
        assert_eq!(feed_name, "AI feed");
    }
}
