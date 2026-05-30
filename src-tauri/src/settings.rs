use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppSettings {
    pub feed_refresh_debounce_minutes: u64,
    pub auto_update_interval_minutes: u64,
    pub log_level: String,
    #[serde(default)]
    pub last_vacuum: i64,
    #[serde(default)]
    pub default_view_type: String,
    #[serde(default)]
    pub default_view_id: i64,
    #[serde(default)]
    pub auto_collapse_folders: bool,
    #[serde(default)]
    pub mark_feed_read_on_exit: bool,
    #[serde(default)]
    pub article_title_font: String,
    #[serde(default)]
    pub article_body_font: String,
    #[serde(default)]
    pub article_title_color: String,
    #[serde(default)]
    pub article_body_color: String,
    #[serde(default)]
    pub article_bg_color: String,
    #[serde(default)]
    pub show_thumbnails: bool,
    #[serde(default = "default_thumbnail_size")]
    pub thumbnail_size: u64,
    #[serde(default = "default_retention_days")]
    pub article_retention_days: u64,
}

fn default_thumbnail_size() -> u64 {
    56
}
fn default_retention_days() -> u64 {
    90
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            feed_refresh_debounce_minutes: 4,
            auto_update_interval_minutes: 30,
            log_level: "info".to_string(),
            last_vacuum: 0,
            default_view_type: "latest".to_string(),
            default_view_id: -1,
            auto_collapse_folders: true,
            mark_feed_read_on_exit: false,
            article_title_font: String::new(),
            article_body_font: String::new(),
            article_title_color: String::new(),
            article_body_color: String::new(),
            article_bg_color: String::new(),
            show_thumbnails: false,
            thumbnail_size: 56,
            article_retention_days: 90,
        }
    }
}

pub fn load_settings(app_dir: &Path) -> AppSettings {
    let settings_path = app_dir.join("settings.toml");

    if settings_path.exists() {
        let content = fs::read_to_string(&settings_path).unwrap_or_default();
        if let Ok(settings) = toml::from_str(&content) {
            return settings;
        }
    }

    let settings = AppSettings::default();
    save_settings(app_dir, &settings);

    settings
}

pub fn save_settings(app_dir: &Path, settings: &AppSettings) {
    let settings_path = app_dir.join("settings.toml");
    if let Ok(toml_string) = toml::to_string_pretty(settings) {
        let _ = fs::write(settings_path, toml_string);
    }
}
