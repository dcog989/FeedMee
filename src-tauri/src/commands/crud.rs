use std::path::Path;

use crate::{
    AppState, db,
    models::{Article, Folder, Tag},
};
use log::info;
use rusqlite::Connection;
use tauri::{Manager, State};

/// Longest tag name the UI can show without breaking a row.
const MAX_TAG_NAME_CHARS: usize = 64;

/// Mirrors the download cap: an OPML file is a list of names and URLs, so a
/// bigger one is a wrong pick, not a real export.
const MAX_OPML_BYTES: u64 = 10 * 1024 * 1024;

fn normalize_tag_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Tag name cannot be empty.".to_string());
    }
    if name.chars().count() > MAX_TAG_NAME_CHARS {
        return Err(format!("Tag name is limited to {MAX_TAG_NAME_CHARS} characters."));
    }
    Ok(name)
}

#[tauri::command(async)]
pub fn get_folders_with_feeds(state: State<'_, AppState>) -> Result<Vec<Folder>, String> {
    let conn = state.db.lock().unwrap();
    db::get_folders_with_feeds(&conn).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn get_feed_unread_count(feed_id: i64, state: State<'_, AppState>) -> Result<i64, String> {
    let conn = state.db.lock().unwrap();
    db::get_feed_unread_count(&conn, feed_id).map_err(|e| e.to_string())
}

#[tauri::command(async)]
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

#[tauri::command(async)]
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

#[tauri::command(async)]
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

#[tauri::command(async)]
pub fn get_saved_articles(
    limit: usize,
    offset: usize,
    sort_desc: bool,
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    let conn = state.db.lock().unwrap();
    db::get_saved_articles(&conn, limit, offset, sort_desc).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn create_folder(name: String, state: State<'_, AppState>) -> Result<i64, String> {
    let conn = state.db.lock().unwrap();
    db::create_folder(&conn, &name).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn mark_article_saved(id: i64, is_saved: bool, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::update_article_saved(&conn, id, is_saved).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn mark_article_read(id: i64, read: bool, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::set_article_read(&conn, id, read).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn mark_all_read(target_type: String, id: i64, state: State<'_, AppState>) -> Result<(), String> {
    info!("Mark All Read: type={}, id={}", target_type, id);
    let conn = state.db.lock().unwrap();
    mark_read(&conn, &target_type, id)
}

/// String dispatch kept out of the Tauri wrapper so the target mapping is testable.
fn mark_read(conn: &Connection, target_type: &str, id: i64) -> Result<(), String> {
    match target_type {
        "feed" => db::mark_feed_read(conn, id),
        "folder" => db::mark_folder_read(conn, id),
        "global" => db::mark_global_read(conn),
        _ => return Err("Invalid type".to_string()),
    }
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_opml(path: String, app: tauri::AppHandle) -> Result<Vec<i64>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        import_opml_blocking(&path, &state)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Insert an OPML feed unless the same URL is already subscribed, including
/// another outline earlier in the same file. Returns the new feed id.
fn insert_opml_feed(
    conn: &Connection,
    name: &str,
    url: &str,
    declared_type: Option<&str>,
    scope: &mut FolderScope,
) -> Result<Option<i64>, String> {
    let Some((url, feed_type)) = normalize_opml_feed(url, declared_type) else {
        return Ok(None);
    };
    // The rest of the app refuses an address it could never fetch; an import
    // must not be the one path that stores a dead feed.
    if feed_type != "bluesky" && !crate::net::is_public_url(&url) {
        return Ok(None);
    }
    if db::feed_exists_by_url(conn, &url, None).map_err(|e| e.to_string())? {
        return Ok(None);
    }
    // Resolve last: a skipped feed must not create the folder it would have used.
    let folder_id = scope.resolve(conn)?;
    db::create_feed(conn, name, &url, folder_id, &feed_type)
        .map(Some)
        .map_err(|e| e.to_string())
}

/// Map an outline URL to the URL and source type FeedMee stores. Bluesky URLs
/// identify themselves; the outline's declared type only separates website
/// feeds from plain RSS. Readers spell the same HTTP feed as `feed://host/…`,
/// `feed:https://…` or plain http(s), sometimes with padding.
fn normalize_opml_feed(url: &str, declared_type: Option<&str>) -> Option<(String, String)> {
    let url = url.trim();
    if let Some(handle) = url.strip_prefix("bsky:")
        && !handle.trim().is_empty()
    {
        return Some((format!("bsky:{}", handle.trim()), "bluesky".to_string()));
    }
    let url = if let Some(rest) = url.strip_prefix("feed:") {
        let rest = rest.trim();
        if rest.starts_with("http://") || rest.starts_with("https://") {
            rest.to_string()
        } else {
            format!("https://{}", rest.trim_start_matches('/'))
        }
    } else {
        url.to_string()
    };
    let is_http = url.starts_with("http://") || url.starts_with("https://");
    if !is_http {
        return None;
    }
    let feed_type = if declared_type == Some("website") {
        "website"
    } else {
        "rss"
    };
    Some((url, feed_type.to_string()))
}

/// Folder an outline belongs to. A top-level folder is created on the first
/// feed that actually lands in it, so an import that adds nothing leaves no
/// empty folder behind.
#[derive(Default)]
struct FolderScope<'a> {
    id: Option<i64>,
    pending_name: Option<&'a str>,
}

impl<'a> FolderScope<'a> {
    fn is_open(&self) -> bool {
        self.id.is_some() || self.pending_name.is_some()
    }

    fn resolve(&mut self, conn: &Connection) -> Result<Option<i64>, String> {
        if self.id.is_none()
            && let Some(name) = self.pending_name.take()
        {
            self.id = Some(db::create_folder(conn, name).map_err(|e| e.to_string())?);
        }
        Ok(self.id)
    }
}

fn insert_opml_outline(
    conn: &Connection,
    outline: &opml::Outline,
    scope: &mut FolderScope,
    imported: &mut Vec<i64>,
) -> Result<(), String> {
    if let Some(url) = &outline.xml_url {
        if let Some(feed_id) = insert_opml_feed(conn, &outline.text, url, outline.r#type.as_deref(), scope)? {
            imported.push(feed_id);
        }
        return Ok(());
    }
    // The app has one folder level, so a nested folder collapses into its parent.
    if scope.is_open() {
        for child in &outline.outlines {
            insert_opml_outline(conn, child, scope, imported)?;
        }
        return Ok(());
    }
    let mut folder = FolderScope {
        id: None,
        pending_name: Some(&outline.text),
    };
    for child in &outline.outlines {
        insert_opml_outline(conn, child, &mut folder, imported)?;
    }
    Ok(())
}

fn read_opml_file(path: &str) -> Result<String, String> {
    let size = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    if size > MAX_OPML_BYTES {
        return Err(format!("OPML file is larger than {MAX_OPML_BYTES} bytes"));
    }
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

fn import_opml_blocking(path: &str, state: &AppState) -> Result<Vec<i64>, String> {
    let xml_content = read_opml_file(path)?;
    let document = opml::OPML::from_str(&xml_content).map_err(|e| e.to_string())?;
    let conn = state.db.lock().unwrap();
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    let mut imported = Vec::new();
    for outline in &document.body.outlines {
        insert_opml_outline(&tx, outline, &mut FolderScope::default(), &mut imported)?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(imported)
}

#[tauri::command]
pub async fn export_opml(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let folders = {
        let conn = state.db.lock().unwrap();
        db::get_folders_with_feeds(&conn).map_err(|e| e.to_string())?
    };

    let opml = crate::opml::render(&folders)?;
    write_opml_file(&path, opml)
}

/// Write beside the target, then rename, so a failed write cannot leave the
/// previous export truncated.
fn write_opml_file(path: &str, opml: String) -> Result<(), String> {
    let temp = Path::new(path).with_extension("opml.tmp");
    let written = std::fs::write(&temp, opml).and_then(|()| std::fs::rename(&temp, path));
    if let Err(e) = written {
        let _ = std::fs::remove_file(&temp);
        return Err(format!("Failed to write {}: {}", path, e));
    }
    Ok(())
}

#[tauri::command(async)]
pub fn rename_folder(id: i64, new_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    if db::folder_name_exists(&conn, &new_name, Some(id)).map_err(|e| e.to_string())? {
        return Err("A folder with that name already exists".to_string());
    }
    db::rename_folder(&conn, id, &new_name).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn rename_feed(id: i64, new_name: String, new_url: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    let feed = db::get_feed(&conn, id).map_err(|e| e.to_string())?;
    if !rename_url_is_valid(&feed.feed_type, &new_url) {
        return Err("Invalid feed URL".to_string());
    }
    if db::feed_exists_by_url(&conn, &new_url, Some(id)).map_err(|e| e.to_string())? {
        return Err("Feed already exists".to_string());
    }
    db::rename_feed(&conn, id, &new_name, &new_url).map_err(|e| e.to_string())
}

/// A renamed URL must keep the feed's own scheme: Bluesky feeds keep a
/// non-empty `bsky:` handle, everything else must stay a public http(s) address.
fn rename_url_is_valid(feed_type: &str, new_url: &str) -> bool {
    if feed_type == "bluesky" {
        new_url
            .strip_prefix("bsky:")
            .is_some_and(|handle| !handle.trim().is_empty())
    } else {
        crate::net::is_public_url(new_url)
    }
}

#[tauri::command(async)]
pub fn delete_feed(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::delete_feed(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn delete_folder(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::delete_folder(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn move_feed(feed_id: i64, folder_id: Option<i64>, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::move_feed(&conn, feed_id, folder_id).map_err(|e| e.to_string())
}

#[tauri::command(async)]
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

#[tauri::command(async)]
pub fn get_tags_for_article(article_id: i64, state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let conn = state.db.lock().unwrap();
    db::get_tags_for_article(&conn, article_id).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let conn = state.db.lock().unwrap();
    db::get_all_tags(&conn).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn add_tag(article_id: i64, name: String, state: State<'_, AppState>) -> Result<Tag, String> {
    let name = normalize_tag_name(&name)?;
    let conn = state.db.lock().unwrap();
    db::add_tag_to_article(&conn, article_id, name).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn remove_tag(article_id: i64, tag_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::remove_tag_from_article(&conn, article_id, tag_id).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn delete_tag(tag_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::delete_tag(&conn, tag_id).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        FolderScope, MAX_OPML_BYTES, insert_opml_feed, insert_opml_outline, mark_read, normalize_tag_name,
        read_opml_file, rename_url_is_valid, write_opml_file,
    };
    use crate::{db, models::Article};
    use rusqlite::Connection;

    /// Unique per test so parallel runs cannot share a file.
    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("feedmee-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Test shorthand for a feed with a known folder.
    fn insert_feed(
        conn: &Connection,
        name: &str,
        url: &str,
        folder_id: Option<i64>,
        declared_type: Option<&str>,
    ) -> Result<Option<i64>, String> {
        let mut scope = FolderScope {
            id: folder_id,
            pending_name: None,
        };
        insert_opml_feed(conn, name, url, declared_type, &mut scope)
    }

    #[test]
    fn opml_import_skips_a_feed_url_that_already_exists() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::init_db(&mut conn).unwrap();

        insert_feed(&conn, "One", "https://example.com/feed", None, None).unwrap();
        insert_feed(&conn, "Two", "https://example.com/feed", None, None).unwrap();

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

        insert_feed(&conn, "One", "feed://example.com/rss", None, None).unwrap();

        let url: String = conn
            .query_row("SELECT url FROM feeds WHERE name = 'One'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(url, "https://example.com/rss");
    }

    #[test]
    fn opml_import_skips_non_http_urls() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::init_db(&mut conn).unwrap();

        insert_feed(&conn, "Bad", "ftp://example.com/rss", None, None).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM feeds WHERE name = 'Bad'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn opml_import_keeps_non_rss_types() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::init_db(&mut conn).unwrap();

        insert_feed(&conn, "Bsky", "bsky:did:plc:abc", None, Some("bluesky")).unwrap();
        insert_feed(&conn, "Site", "https://example.com/", None, Some("website")).unwrap();

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
        insert_opml_outline(&conn, &folder, &mut FolderScope::default(), &mut Vec::new()).unwrap();

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

    #[test]
    fn tag_names_are_trimmed_and_bounded() {
        assert_eq!(normalize_tag_name("  News  ").unwrap(), "News");
        assert!(normalize_tag_name("   ").is_err());
        assert!(normalize_tag_name(&"x".repeat(65)).is_err());
    }

    #[test]
    fn opml_import_accepts_feed_scheme_and_padding() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::init_db(&mut conn).unwrap();

        insert_feed(&conn, "Secure", "feed:https://example.com/secure", None, None).unwrap();
        insert_feed(&conn, "Padded", "  https://example.com/padded  ", None, None).unwrap();

        let url = |name: &str| -> String {
            conn.query_row("SELECT url FROM feeds WHERE name = ?1", [name], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(url("Secure"), "https://example.com/secure");
        assert_eq!(url("Padded"), "https://example.com/padded");
    }

    #[test]
    fn opml_import_skips_feeds_the_app_cannot_fetch() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::init_db(&mut conn).unwrap();

        insert_feed(&conn, "LAN", "http://192.168.1.10/feed", None, None).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM feeds WHERE name = 'LAN'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn opml_import_does_not_create_folders_without_feeds() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::init_db(&mut conn).unwrap();
        insert_feed(&conn, "Existing", "https://example.com/feed", None, None).unwrap();

        let folder = opml::Outline {
            text: "Tech".to_string(),
            outlines: vec![opml::Outline {
                text: "Duplicate".to_string(),
                xml_url: Some("https://example.com/feed".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut imported = Vec::new();
        insert_opml_outline(&conn, &folder, &mut FolderScope::default(), &mut imported).unwrap();

        assert!(imported.is_empty());
        let folders: i64 = conn
            .query_row("SELECT COUNT(*) FROM folders WHERE id != 0", [], |r| r.get(0))
            .unwrap();
        assert_eq!(folders, 0);
    }

    #[test]
    fn opml_import_reports_the_feeds_it_added() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::init_db(&mut conn).unwrap();

        let folder = opml::Outline {
            text: "Tech".to_string(),
            outlines: vec![opml::Outline {
                text: "Feed".to_string(),
                xml_url: Some("https://example.com/feed".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut imported = Vec::new();
        insert_opml_outline(&conn, &folder, &mut FolderScope::default(), &mut imported).unwrap();

        assert_eq!(imported.len(), 1);
        let folder_id: i64 = conn
            .query_row("SELECT folder_id FROM feeds WHERE id = ?1", [imported[0]], |r| r.get(0))
            .unwrap();
        assert_ne!(folder_id, 0, "the feed must land in the folder the file asked for");
    }

    #[test]
    fn opml_files_larger_than_the_limit_are_refused_before_reading() {
        let dir = temp_dir("opml-size");
        let path = dir.join("huge.opml");
        std::fs::File::create(&path)
            .unwrap()
            .set_len(MAX_OPML_BYTES + 1)
            .unwrap();

        assert!(read_opml_file(path.to_str().unwrap()).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_failed_export_keeps_the_previous_file() {
        let dir = temp_dir("export-fail");
        let target = dir.join("feeds.opml");
        std::fs::write(&target, "old").unwrap();
        // A directory at the temp path makes the write fail before the rename.
        std::fs::create_dir_all(dir.join("feeds.opml.tmp")).unwrap();

        assert!(write_opml_file(target.to_str().unwrap(), "new".to_string()).is_err());

        let kept = std::fs::read_to_string(&target).unwrap();
        assert_eq!(kept, "old");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_successful_export_replaces_the_file_and_leaves_no_temp() {
        let dir = temp_dir("export-ok");
        let target = dir.join("feeds.opml");
        std::fs::write(&target, "old").unwrap();

        write_opml_file(target.to_str().unwrap(), "new".to_string()).unwrap();

        assert_eq!(std::fs::read_to_string(&target).unwrap(), "new");
        assert!(!dir.join("feeds.opml.tmp").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mark_read_dispatches_to_the_named_target() {
        let mut conn = Connection::open_in_memory().unwrap();
        db::init_db(&mut conn).unwrap();
        let folder = db::create_folder(&conn, "News").unwrap();
        let inside = db::create_feed(&conn, "Inside", "https://example.com/in", Some(folder), "rss").unwrap();
        let outside = db::create_feed(&conn, "Outside", "https://example.com/out", None, "rss").unwrap();

        for (feed_id, url) in [(inside, "https://example.com/a"), (outside, "https://example.com/b")] {
            let article = Article {
                id: 0,
                feed_id,
                title: "Title".to_string(),
                author: String::new(),
                summary: String::new(),
                url: url.to_string(),
                image_url: String::new(),
                image_low_res: false,
                timestamp: 1,
                is_read: false,
                is_saved: false,
                has_tags: false,
            };
            db::batch_insert_articles(&mut conn, &[article], None).unwrap();
        }

        mark_read(&conn, "folder", folder).unwrap();

        assert_eq!(db::get_feed_unread_count(&conn, inside).unwrap(), 0);
        assert_eq!(db::get_feed_unread_count(&conn, outside).unwrap(), 1);
        assert!(mark_read(&conn, "bogus", 0).is_err());
    }

    #[test]
    fn rename_url_rules_match_the_feed_type() {
        assert!(rename_url_is_valid("rss", "https://example.com/feed"));
        assert!(!rename_url_is_valid("rss", "http://127.0.0.1/feed"));
        assert!(rename_url_is_valid("bluesky", "bsky:did:plc:abc"));
        assert!(!rename_url_is_valid("bluesky", "bsky:"));
        assert!(!rename_url_is_valid("bluesky", "bsky:   "));
        assert!(!rename_url_is_valid("bluesky", "https://bsky.app/profile/example.com"));
    }
}
