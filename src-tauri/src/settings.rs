use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppSettings {
    pub feed_refresh_debounce_minutes: u64,
    pub refresh_all_debounce_minutes: u64,
    pub auto_update_interval_minutes: u64,
    pub log_level: String,
    #[serde(default)] // Allow missing field for backward compatibility
    pub last_vacuum: i64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            feed_refresh_debounce_minutes: 5,
            refresh_all_debounce_minutes: 2,
            auto_update_interval_minutes: 30,
            log_level: "info".to_string(),
            last_vacuum: 0,
        }
    }
}

pub fn load_settings(app_dir: &PathBuf) -> AppSettings {
    let settings_path = app_dir.join("settings.toml");

    if settings_path.exists() {
        let content = fs::read_to_string(&settings_path).unwrap_or_default();
        if let Ok(settings) = toml::from_str(&content) {
            return settings;
        }
    }

    // Write defaults if missing or invalid
    let settings = AppSettings::default();
    save_settings(app_dir, &settings);

    settings
}

pub fn save_settings(app_dir: &PathBuf, settings: &AppSettings) {
    let settings_path = app_dir.join("settings.toml");
    if let Ok(toml_string) = toml::to_string_pretty(settings) {
        let _ = fs::write(settings_path, toml_string);
    }
}
