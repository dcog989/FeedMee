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

/// Write a file beside its destination and rename it into place, so a crash
/// leaves the old file rather than a truncated new one.
pub(crate) fn write_atomic(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    use std::io::Write;

    let mut temp = path.as_os_str().to_owned();
    temp.push(".tmp");
    let temp = PathBuf::from(temp);

    let mut file = fs::File::create(&temp)?;
    file.write_all(contents)?;
    file.sync_all()?;
    drop(file);
    // The directory entry is not fsynced: a power cut can lose the last write,
    // never the file that was already there.
    fs::rename(&temp, path)
}

/// What was moved aside when the old library could not be used.
pub(crate) enum Recovery {
    /// SQLite says the file is not a database.
    Corrupt(PathBuf),
    /// The schema was written by a release this build cannot upgrade.
    Unsupported(PathBuf),
}

/// Open the library, moving the file aside only when SQLite says it is not a
/// database or its schema cannot be upgraded. Disk, permission and lock errors
/// keep the file: those are not corruption, and replacing the library would
/// look like data loss.
///
/// Returns the connection and, when the old file could not be used, what was
/// moved aside, so the caller can tell the user.
pub(crate) fn setup_database(db_path: &Path) -> Result<(rusqlite::Connection, Option<Recovery>), String> {
    match open_database(db_path) {
        Ok(conn) => Ok((conn, None)),
        Err(e) if is_corruption(e.as_ref()) => {
            log::error!("Database is unreadable ({}). Moving it aside and starting fresh.", e);
            let moved_to = quarantine_database(db_path, "corrupt")?;
            let conn = open_database(db_path).map_err(|e| format!("Failed to create a fresh database: {}", e))?;
            Ok((conn, Some(Recovery::Corrupt(moved_to))))
        },
        Err(e) if e.downcast_ref::<db::UnsupportedSchema>().is_some() => {
            log::error!(
                "Database schema cannot be upgraded ({}). Moving it aside and starting fresh.",
                e
            );
            let moved_to = quarantine_database(db_path, "pre-baseline")?;
            let conn = open_database(db_path).map_err(|e| format!("Failed to create a fresh database: {}", e))?;
            Ok((conn, Some(Recovery::Unsupported(moved_to))))
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

fn quarantine_database(db_path: &Path, tag: &str) -> Result<PathBuf, String> {
    let stamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let moved_db = PathBuf::from(format!("{}.{}-{}", db_path.display(), tag, stamp));
    for suffix in ["", "-wal", "-shm"] {
        let from = PathBuf::from(format!("{}{}", db_path.display(), suffix));
        if !from.exists() {
            continue;
        }
        let to = if suffix.is_empty() {
            moved_db.clone()
        } else {
            PathBuf::from(format!("{}.{}-{}", from.display(), tag, stamp))
        };
        fs::rename(&from, &to).map_err(|e| format!("Failed to move the database aside: {}", e))?;
    }
    Ok(moved_db)
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

#[cfg(test)]
mod tests {
    use super::{Recovery, setup_database};
    use std::fs;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("feedmee-startup-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn user_version(conn: &rusqlite::Connection) -> i64 {
        conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap()
    }

    #[test]
    fn corrupt_database_is_moved_aside_and_replaced() {
        let dir = temp_dir("corrupt");
        let db_path = dir.join("feedmee.db");
        fs::write(&db_path, b"this is not a database, no matter how it is read").unwrap();

        let (conn, recovered) = setup_database(&db_path).expect("a fresh database must open");
        let Some(Recovery::Corrupt(moved_to)) = recovered else {
            panic!("the unreadable file must be reported");
        };
        assert_eq!(
            fs::read(&moved_to).unwrap(),
            b"this is not a database, no matter how it is read".to_vec()
        );

        assert_eq!(user_version(&conn), 2);

        drop(conn);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unsupported_schema_is_moved_aside_and_replaced() {
        let dir = temp_dir("unsupported");
        let db_path = dir.join("feedmee.db");
        {
            // A library stamped by a pre-baseline release, without its columns.
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.pragma_update(None, "user_version", 9).unwrap();
        }

        let (conn, recovered) = setup_database(&db_path).expect("a fresh database must open");
        let Some(Recovery::Unsupported(moved_to)) = recovered else {
            panic!("the unsupported file must be reported");
        };
        assert!(moved_to.exists());
        assert!(moved_to.to_string_lossy().contains("pre-baseline"));
        assert_eq!(user_version(&conn), 2);

        drop(conn);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unopenable_database_is_kept_in_place() {
        let dir = temp_dir("unopenable");
        let db_path = dir.join("feedmee.db");
        // A directory where the file should be: SQLite cannot open it, but that
        // is not corruption, so nothing may be moved aside.
        fs::create_dir_all(&db_path).unwrap();

        assert!(setup_database(&db_path).is_err());
        assert!(db_path.is_dir());

        let _ = fs::remove_dir_all(&dir);
    }
}
