use crate::AppState;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

#[derive(Serialize)]
pub struct StorageStats {
    pub db_size_bytes: u64,
    pub cache_size_bytes: u64,
    pub cache_file_count: u64,
}

fn dir_stats(dir: &Path) -> (u64, u64) {
    let mut bytes = 0;
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata()
                && metadata.is_file()
            {
                bytes += metadata.len();
                count += 1;
            }
        }
    }
    (bytes, count)
}

#[tauri::command]
pub fn get_storage_stats(state: State<'_, AppState>) -> StorageStats {
    let (cache_size_bytes, cache_file_count) = dir_stats(&state.paths.thumbnail_dir());
    let db_size_bytes = fs::metadata(state.paths.db_path()).map(|m| m.len()).unwrap_or(0);

    StorageStats {
        db_size_bytes,
        cache_size_bytes,
        cache_file_count,
    }
}

#[tauri::command]
pub fn clear_thumbnail_cache(state: State<'_, AppState>) -> u64 {
    let mut freed = 0;
    if let Ok(entries) = fs::read_dir(state.paths.thumbnail_dir()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            if fs::remove_file(&path).is_ok() {
                freed += size;
            }
        }
    }
    if freed > 0 {
        log::info!("Cleared thumbnail cache ({} bytes)", freed);
    }
    freed
}

#[tauri::command]
pub fn vacuum_database(state: State<'_, AppState>) -> Result<(), String> {
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        crate::db::run_vacuum(&conn).map_err(|e| e.to_string())?;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.last_vacuum = now;
    crate::settings::save_settings(&state.paths.config_dir, &settings);
    Ok(())
}
