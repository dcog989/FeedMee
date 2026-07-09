use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::db;
use crate::paths;

const MAX_BACKUPS: usize = 24;

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn generate_opml(conn: &rusqlite::Connection) -> Result<String, String> {
    let folders = db::get_folders_with_feeds(conn).map_err(|e| e.to_string())?;

    let mut opml = String::new();
    use std::fmt::Write;
    let _ = writeln!(&mut opml, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    let _ = writeln!(&mut opml, r#"<opml version="2.0">"#);
    let _ = writeln!(&mut opml, r#"  <head><title>FeedMee Export</title></head>"#);
    let _ = writeln!(&mut opml, r#"  <body>"#);

    for folder in folders {
        if folder.feeds.is_empty() {
            continue;
        }
        if folder.id == 0 {
            for feed in &folder.feeds {
                let _ = writeln!(
                    &mut opml,
                    r#"      <outline type="rss" text="{}" xmlUrl="{}" />"#,
                    xml_escape(&feed.name),
                    xml_escape(&feed.url)
                );
            }
        } else {
            let _ = writeln!(
                &mut opml,
                r#"    <outline text="{}">"#,
                xml_escape(&folder.name)
            );
            for feed in &folder.feeds {
                let _ = writeln!(
                    &mut opml,
                    r#"      <outline type="rss" text="{}" xmlUrl="{}" />"#,
                    xml_escape(&feed.name),
                    xml_escape(&feed.url)
                );
            }
            let _ = writeln!(&mut opml, r#"    </outline>"#);
        }
    }
    let _ = writeln!(&mut opml, r#"  </body>"#);
    let _ = writeln!(&mut opml, r#"</opml>"#);
    Ok(opml)
}

fn backup_dir() -> PathBuf {
    paths::config_dir().join("Backup")
}

fn rotate_backups(dir: &Path) {
    let mut entries: Vec<PathBuf> = match fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "opml"))
            .map(|e| e.path())
            .collect(),
        Err(_) => return,
    };

    entries.sort();

    while entries.len() >= MAX_BACKUPS {
        if let Some(oldest) = entries.first() {
            let _ = fs::remove_file(oldest);
            entries.remove(0);
        }
    }
}

pub fn run_auto_backup(db: &Mutex<rusqlite::Connection>) -> Result<(), String> {
    let dir = backup_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("failed to create backup dir: {}", e))?;

    let conn = db.lock().map_err(|e| format!("db lock: {}", e))?;
    let opml = generate_opml(&conn)?;
    drop(conn);

    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H%M%S");
    let filename = format!("backup-{}.opml", timestamp);
    let filepath = dir.join(&filename);

    fs::write(&filepath, &opml).map_err(|e| format!("failed to write backup: {}", e))?;

    log::info!("Auto-backup written: {}", filepath.display());

    rotate_backups(&dir);

    Ok(())
}
