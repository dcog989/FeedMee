use crate::{AppState, db, models::{Article, Folder, Tag}};
use log::info;
use std::fmt::Write;
use tauri::State;

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
    db::get_articles_for_folder(&conn, folder_id, limit, offset, sort_desc)
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
    db::get_latest_articles(&conn, cutoff_timestamp, limit, offset, sort_desc)
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
    db::get_saved_articles(&conn, limit, offset, sort_desc).map_err(|e| e.to_string())
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
    } else if target_type == "global" {
        db::mark_global_read(&conn).map_err(|e| e.to_string())
    } else {
        Err("Invalid type".to_string())
    }
}

#[tauri::command]
pub async fn import_opml(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let xml_content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let document = opml::OPML::from_str(&xml_content).map_err(|e| e.to_string())?;
    let conn = state.db.lock().unwrap();
    let mut flat_feeds: Vec<(String, String)> = Vec::new();

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
            flat_feeds.push((outline.text, url));
        }
    }

    if !flat_feeds.is_empty()
        && let Ok(default_folder_id) = db::create_folder(&conn, "Uncategorized")
    {
        for (name, url) in flat_feeds {
            let _ = db::create_feed(&conn, &name, &url, default_folder_id, "rss");
        }
    }
    Ok(())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[tauri::command]
pub async fn export_opml(state: State<'_, AppState>) -> Result<String, String> {
    let folders = {
        let conn = state.db.lock().unwrap();
        db::get_folders_with_feeds(&conn).map_err(|e| e.to_string())?
    };

    let mut opml = String::new();
    let _ = writeln!(&mut opml, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    let _ = writeln!(&mut opml, "<opml version=\"2.0\">");
    let _ = writeln!(&mut opml, "  <head><title>FeedMee Export</title></head>");
    let _ = writeln!(&mut opml, "  <body>");

    for folder in folders {
        if folder.feeds.is_empty() {
            continue;
        }
        let _ = writeln!(&mut opml, "    <outline text=\"{}\">", xml_escape(&folder.name));
        for feed in &folder.feeds {
            let _ = writeln!(
                &mut opml,
                "      <outline type=\"rss\" text=\"{}\" xmlUrl=\"{}\" />",
                xml_escape(&feed.name),
                xml_escape(&feed.url)
            );
        }
        let _ = writeln!(&mut opml, "    </outline>");
    }
    let _ = writeln!(&mut opml, "  </body>");
    let _ = writeln!(&mut opml, "</opml>");
    Ok(opml)
}

#[tauri::command]
pub async fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_folder(id: i64, new_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::rename_folder(&conn, id, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_feed(id: i64, new_name: String, new_url: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
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
pub fn add_tag(
    article_id: i64,
    name: String,
    color: String,
    state: State<'_, AppState>,
) -> Result<Tag, String> {
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
