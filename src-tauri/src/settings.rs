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
            thumbnail_size: 56,
            article_retention_days: 90,
        }
    }
}

pub fn load_settings(config_dir: &Path) -> AppSettings {
    fs::create_dir_all(config_dir).ok();
    let path = config_dir.join("settings.toml");

    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(settings) => return settings,
                Err(e) => {
                    eprintln!(
                        "[settings] failed to parse {}: {}. Keeping file and using defaults.",
                        path.display(),
                        e
                    );
                    return AppSettings::default();
                },
            },
            Err(e) => {
                eprintln!("[settings] failed to read {}: {}. Using defaults.", path.display(), e);
                return AppSettings::default();
            },
        }
    }

    let settings = AppSettings::default();
    let _ = save_settings(config_dir, &settings);
    settings
}

pub fn save_settings(config_dir: &Path, settings: &AppSettings) -> Result<(), String> {
    fs::create_dir_all(config_dir).map_err(|e| format!("Failed to create config directory: {}", e))?;
    let toml_string = toml::to_string_pretty(settings).map_err(|e| format!("Failed to serialize settings: {}", e))?;
    let path = config_dir.join("settings.toml");
    let temp = config_dir.join("settings.toml.tmp");
    fs::write(&temp, toml_string).map_err(|e| format!("Failed to write settings: {}", e))?;
    fs::rename(&temp, &path).map_err(|e| format!("Failed to replace settings: {}", e))
}

#[cfg(test)]
mod tests {
    use super::{AppSettings, load_settings, save_settings};

    #[test]
    fn settings_survive_a_save_and_load() {
        let dir = std::env::temp_dir().join(format!("feedmee-settings-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let mut settings = AppSettings::default();
        settings.log_level = "debug".to_string();
        settings.article_retention_days = 7;
        save_settings(&dir, &settings).unwrap();

        let loaded = load_settings(&dir);
        assert_eq!(loaded.log_level, "debug");
        assert_eq!(loaded.article_retention_days, 7);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
