use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::db;
use crate::paths;

const MAX_BACKUPS: usize = 24;

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

    while entries.len() > MAX_BACKUPS {
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
    let folders = db::get_folders_with_feeds(&conn).map_err(|e| e.to_string())?;
    drop(conn);
    let opml = crate::opml::render(&folders);

    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H%M%S");
    let filename = format!("backup-{}.opml", timestamp);
    let filepath = dir.join(&filename);

    fs::write(&filepath, &opml).map_err(|e| format!("failed to write backup: {}", e))?;

    log::info!("Auto-backup written: {}", filepath.display());

    rotate_backups(&dir);

    Ok(())
}
