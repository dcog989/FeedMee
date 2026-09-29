use std::path::PathBuf;

use tauri::{AppHandle, Manager, Runtime};

const SHORTCUTS_FILENAME: &str = "shortcuts.json";
const DATABASE_DIRNAME: &str = "Database";
const BACKUP_DIRNAME: &str = "Backup";
const THUMBNAIL_DIRNAME: &str = "thumbnails";

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AppPaths {
    pub fn resolve<R: Runtime>(app: &AppHandle<R>) -> Result<Self, String> {
        let resolver = app.path();
        Ok(Self {
            config_dir: resolver.app_config_dir().map_err(|e| e.to_string())?,
            data_dir: resolver.app_local_data_dir().map_err(|e| e.to_string())?,
            cache_dir: resolver.app_cache_dir().map_err(|e| e.to_string())?,
            logs_dir: resolver.app_log_dir().map_err(|e| e.to_string())?,
        })
    }

    pub fn shortcuts_path(&self) -> PathBuf {
        self.config_dir.join(SHORTCUTS_FILENAME)
    }

    pub fn db_dir(&self) -> PathBuf {
        self.data_dir.join(DATABASE_DIRNAME)
    }

    pub fn db_path(&self) -> PathBuf {
        self.db_dir().join(crate::db::DB_FILENAME)
    }

    pub fn backup_dir(&self) -> PathBuf {
        self.data_dir.join(BACKUP_DIRNAME)
    }

    pub fn thumbnail_dir(&self) -> PathBuf {
        self.cache_dir.join(THUMBNAIL_DIRNAME)
    }

    pub fn create_dirs(&self) -> std::io::Result<()> {
        for dir in [
            &self.config_dir,
            &self.data_dir,
            &self.cache_dir,
            &self.logs_dir,
            &self.db_dir(),
            &self.backup_dir(),
            &self.thumbnail_dir(),
        ] {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }
}
