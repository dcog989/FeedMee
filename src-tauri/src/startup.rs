use std::fs;
use std::path::{Path, PathBuf};

use flexi_logger::{Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, LoggerHandle, Naming};

use crate::db;

const LOG_RETENTION_DAYS: usize = 5;
const LOG_FILE_PREFIX: &str = "feedmee";
const DEFAULT_LOG_LEVEL: &str = "info";
const MAX_REDIRECTS: usize = 10;
const WINDOW_REVEAL_FALLBACK: std::time::Duration = std::time::Duration::from_secs(3);
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
        .format(flexi_logger::opt_format)
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

/// Open the library, moving the file aside only when SQLite says it is not a
/// database. Disk, permission and lock errors keep the file: those are not
/// corruption, and replacing the library would look like data loss.
pub(crate) fn setup_database(db_path: &Path) -> Result<rusqlite::Connection, String> {
    match open_database(db_path) {
        Ok(conn) => Ok(conn),
        Err(e) if is_corruption(e.as_ref()) => {
            log::error!("Database is unreadable ({}). Moving it aside and starting fresh.", e);
            quarantine_database(db_path)?;
            open_database(db_path).map_err(|e| format!("Failed to create a fresh database: {}", e))
        },
        Err(e) => Err(format!("Could not open database {}: {}", db_path.display(), e)),
    }
}

fn is_corruption(err: &(dyn std::error::Error + 'static)) -> bool {
    err.downcast_ref::<rusqlite::Error>().is_some_and(|e| {
        matches!(
            e,
            rusqlite::Error::SqliteFailure(code, _)
                if matches!(code.code, rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase)
        )
    })
}

fn quarantine_database(db_path: &Path) -> Result<(), String> {
    let stamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    for suffix in ["", "-wal", "-shm"] {
        let from = PathBuf::from(format!("{}{}", db_path.display(), suffix));
        if from.exists() {
            let to = PathBuf::from(format!("{}.corrupt-{}", from.display(), stamp));
            fs::rename(&from, &to).map_err(|e| format!("Failed to move unreadable database aside: {}", e))?;
        }
    }
    Ok(())
}

fn open_database(db_path: &Path) -> Result<rusqlite::Connection, Box<dyn std::error::Error>> {
    let mut conn = rusqlite::Connection::open(db_path)?;
    db::init_db(&mut conn)?;
    Ok(conn)
}

pub(crate) fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(concat!("FeedMee/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            // A redirect can walk around the check on the original URL.
            if attempt.previous().len() >= MAX_REDIRECTS || !crate::net::is_public_url(attempt.url().as_str()) {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .build()
        .expect("failed to build HTTP client")
}

/// Shows the window if the frontend never asked for it. Without this a broken
/// bundle leaves a running process with no window and no way to open it.
pub(crate) fn reveal_window_fallback(window: &tauri::WebviewWindow) {
    let window = window.clone();
    std::thread::spawn(move || {
        std::thread::sleep(WINDOW_REVEAL_FALLBACK);
        if window.is_visible().unwrap_or(true) {
            return;
        }
        log::warn!("Frontend did not reveal the window; showing it anyway");
        let _ = window.show();
    });
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
