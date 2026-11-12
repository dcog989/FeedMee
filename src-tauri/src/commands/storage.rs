use crate::AppState;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
pub struct StorageStats {
    pub db_size_bytes: u64,
    pub cache_size_bytes: u64,
    pub cache_file_count: u64,
}

fn file_len(path: &Path) -> u64 {
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
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
pub async fn get_storage_stats(app: AppHandle) -> Result<StorageStats, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let (cache_size_bytes, cache_file_count) = dir_stats(&state.paths.thumbnail_dir());

        let db_path = state.paths.db_path();
        let wal_path = PathBuf::from(format!("{}-wal", db_path.display()));
        let db_size_bytes = file_len(&db_path) + file_len(&wal_path);

        StorageStats {
            db_size_bytes,
            cache_size_bytes,
            cache_file_count,
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_thumbnail_cache(app: AppHandle) -> Result<u64, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
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
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn vacuum_database(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
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
        crate::settings::save_settings(&state.paths.config_dir, &settings).map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
