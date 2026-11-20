use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::db;

const MAX_BACKUPS: usize = 10;

fn sorted_backup_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "opml"))
        .map(|e| e.path())
        .collect();
    files.sort();
    files
}

/// Deletes the oldest backups so only `MAX_BACKUPS` remain. Names sort
/// chronologically, so plain filename order is age order.
fn prune_backups(dir: &Path) {
    let files = sorted_backup_files(dir);
    let excess = files.len().saturating_sub(MAX_BACKUPS);
    for old in files.iter().take(excess) {
        let _ = fs::remove_file(old);
    }
}

pub fn run_auto_backup(db: &Mutex<rusqlite::Connection>, backup_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(backup_dir).map_err(|e| format!("failed to create backup dir: {}", e))?;

    let conn = db.lock().map_err(|e| format!("db lock: {}", e))?;
    let folders = db::get_folders_with_feeds(&conn).map_err(|e| e.to_string())?;
    drop(conn);
    let opml = crate::opml::render(&folders)?;

    if let Some(latest) = sorted_backup_files(backup_dir).last()
        && fs::read(latest).is_ok_and(|previous| previous == opml.as_bytes())
    {
        // Subscriptions are unchanged, so the last backup already covers this.
        // Still trim, so backups left by older rules do not linger.
        prune_backups(backup_dir);
        return Ok(());
    }

    let timestamp = chrono::Local::now().format("%Y-%m-%dT%H-%M-%S");
    let filename = format!("backup-{}.opml", timestamp);
    let filepath = backup_dir.join(&filename);

    crate::startup::write_atomic(&filepath, opml.as_bytes()).map_err(|e| format!("failed to write backup: {}", e))?;
    log::info!("Auto-backup written: {}", filepath.display());

    prune_backups(backup_dir);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_backups(dir: &Path, count: usize) {
        for i in 1..=count {
            let path = dir.join(format!("backup-2026-01-{:02}T00-00-00.opml", i));
            fs::write(path, "opml").unwrap();
        }
    }

    #[test]
    fn prune_keeps_only_the_newest_backups() {
        let dir = std::env::temp_dir().join(format!("feedmee-backup-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_backups(&dir, MAX_BACKUPS + 2);

        prune_backups(&dir);

        let remaining = sorted_backup_files(&dir);
        assert_eq!(remaining.len(), MAX_BACKUPS);
        let oldest = remaining.first().unwrap().to_string_lossy();
        assert!(oldest.ends_with("2026-01-03T00-00-00.opml"));

        fs::remove_dir_all(&dir).unwrap();
    }
}
