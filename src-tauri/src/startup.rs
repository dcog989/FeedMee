use std::path::Path;

use flexi_logger::{Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, LoggerHandle, Naming};

use crate::db;

const LOG_RETENTION_DAYS: usize = 5;
const LOG_FILE_PREFIX: &str = "feedmee";
const DEFAULT_LOG_LEVEL: &str = "info";
const NOISY_MODULES: [&str; 5] = ["html5ever", "selectors", "scraper", "tendril", "reqwest"];

pub(crate) fn log_spec(level: &str) -> String {
    let level = match level.to_lowercase().as_str() {
        "error" | "warn" | "info" | "debug" | "trace" => level.to_lowercase(),
        _ => DEFAULT_LOG_LEVEL.to_string(),
    };

    let mut spec = level;
    for module in NOISY_MODULES {
        spec.push_str(&format!(",{}=off", module));
    }
    spec
}

pub(crate) fn init_logging(logs_dir: &Path, log_level: &str) -> Result<LoggerHandle, flexi_logger::FlexiLoggerError> {
    Logger::try_with_str(log_spec(log_level))?
        .log_to_file(
            FileSpec::default()
                .directory(logs_dir)
                .basename(LOG_FILE_PREFIX)
                .suffix("log"),
        )
        .rotate(
            Criterion::Age(Age::Day),
            Naming::TimestampsCustomFormat {
                current_infix: Some(""),
                format: "-%Y-%m-%d",
            },
            Cleanup::KeepLogFiles(LOG_RETENTION_DAYS),
        )
        .duplicate_to_stderr(Duplicate::All)
        .append()
        .start()
}

pub(crate) fn setup_database(db_path: &Path) -> rusqlite::Connection {
    let mut conn = rusqlite::Connection::open(db_path).unwrap_or_else(|e| panic!("Failed to open database: {}", e));

    db::init_db(&mut conn).unwrap_or_else(|e| panic!("Schema init failed: {}", e));

    conn
}

pub(crate) fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(concat!("FeedMee/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client")
}

#[cfg(target_os = "linux")]
pub(crate) fn setup_window(window: &tauri::WebviewWindow) {
    use gtk::prelude::GtkWindowExt;

    if let Ok(gtk_window) = window.gtk_window() {
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
        Ok(_) => log::info!("Window icon set successfully"),
        Err(e) => log::warn!("Failed to set window icon: {}", e),
    }
}
