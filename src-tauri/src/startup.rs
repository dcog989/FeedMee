use std::path::{Path, PathBuf};

use simplelog::*;

use crate::db;
use crate::settings::AppSettings;

pub(crate) fn create_dirs() -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let local_dir = crate::paths::local_data_dir();
    let config_dir = crate::paths::config_dir();
    let logs_dir = local_dir.join("Logs");
    let db_dir = config_dir.join("Database");

    for dir in [&local_dir, &config_dir, &logs_dir, &db_dir] {
        if !dir.exists() {
            std::fs::create_dir_all(dir).expect("failed to create app data dir");
        }
    }

    (local_dir, config_dir, logs_dir, db_dir)
}

pub(crate) fn rotate_logs(logs_dir: &Path) {
    let max_logs = 5;
    let oldest = logs_dir.join(format!("feedmee.{}.log", max_logs));
    if oldest.exists() {
        let _ = std::fs::remove_file(oldest);
    }
    for i in (1..max_logs).rev() {
        let current = logs_dir.join(format!("feedmee.{}.log", i));
        let next = logs_dir.join(format!("feedmee.{}.log", i + 1));
        if current.exists() {
            let _ = std::fs::rename(current, next);
        }
    }
    let current_log = logs_dir.join("feedmee.log");
    if current_log.exists() {
        let _ = std::fs::rename(&current_log, logs_dir.join("feedmee.1.log"));
    }
}

pub(crate) fn parse_log_level(level: &str) -> LevelFilter {
    match level.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    }
}

pub(crate) fn init_logging(logs_dir: &Path, log_level: LevelFilter) {
    let log_config = ConfigBuilder::new()
        .add_filter_ignore_str("html5ever")
        .add_filter_ignore_str("selectors")
        .add_filter_ignore_str("scraper")
        .add_filter_ignore_str("tendril")
        .set_time_format_rfc3339()
        .build();

    let log_path = logs_dir.join("feedmee.log");
    let _ = CombinedLogger::init(vec![
        TermLogger::new(
            log_level,
            log_config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            log_level,
            log_config,
            std::fs::File::create(log_path).unwrap(),
        ),
    ]);
}

pub(crate) fn setup_database(
    db_path: &Path,
    _app_settings: &mut AppSettings,
) -> rusqlite::Connection {
    let mut conn = rusqlite::Connection::open(db_path)
        .unwrap_or_else(|e| panic!("Failed to open database: {}", e));

    db::init_db(&mut conn).unwrap_or_else(|e| panic!("Schema init failed: {}", e));

    conn
}

pub(crate) fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client")
}

#[cfg(target_os = "linux")]
pub(crate) fn setup_window(window: &tauri::WebviewWindow) {
    use gtk::prelude::GtkWindowExt;

    if let Ok(gtk_window) = window.gtk_window() {
        gtk_window.set_titlebar(None::<&gtk::Widget>);

        const ICON_BYTES: &[u8] = include_bytes!("../icons/128x128@2x.png");
        if let Ok(img) = image::load_from_memory(ICON_BYTES) {
            let rgba = img.into_rgba8();
            let (w, h) = rgba.dimensions();
            let icon = tauri::image::Image::new_owned(rgba.into_raw(), w, h);
            let _ = window.set_icon(icon);

            if let Ok(pixbuf) = gtk::gdk_pixbuf::Pixbuf::from_read(ICON_BYTES) {
                gtk_window.set_icon(Some(&pixbuf));
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn setup_window(window: &tauri::WebviewWindow) {
    match window.set_icon(tauri::include_image!("icons/32x32.png")) {
        Ok(_) => info!("Window icon set successfully"),
        Err(e) => log::warn!("Failed to set window icon: {}", e),
    }
}
