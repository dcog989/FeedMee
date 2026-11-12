pub mod commands;
pub mod connectors;
pub mod db;
pub mod models;
pub mod net;
pub mod opml;
pub mod paths;
pub mod settings;
pub mod startup;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_window_state::StateFlags;

/// Permits for og:image lookups during refresh. Feed and page fetches are
/// bounded by the frontend's refresh workers, not by this semaphore.
const OG_IMAGE_FETCH_CONCURRENCY: usize = 8;
/// First maintenance pass waits for the first paint, then runs on every launch
/// so short sessions still get a purge and a backup.
const MAINTENANCE_START_DELAY_SECS: u64 = 30;
const MAINTENANCE_INTERVAL: Duration = Duration::from_secs(86400);
const THUMBNAIL_CACHE_MAX_AGE_DAYS: u64 = 7;
const THUMBNAIL_CACHE_MAX_BYTES: u64 = 512 * 1024 * 1024;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub settings: Mutex<settings::AppSettings>,
    pub http_client: reqwest::Client,
    pub og_image_semaphore: Arc<tokio::sync::Semaphore>,
    pub paths: paths::AppPaths,
}

impl AppState {
    /// Cutoff before which timestamped articles are expired, per current
    /// retention settings. `None` when retention is disabled.
    pub fn article_retention_cutoff(&self) -> Option<i64> {
        let days = self.settings.lock().unwrap().article_retention_days;
        db::retention_cutoff(days)
    }
}

/// Purge expired articles and drop stale cached thumbnails. Runs on a worker
/// thread; VACUUM is left to the Settings button because it holds the DB lock
/// for seconds on a large library.
fn run_maintenance(state: &AppState) {
    let retention = state.settings.lock().unwrap().article_retention_days;
    {
        let conn = state.db.lock().unwrap();
        if let Err(e) = db::purge_old_articles(&conn, retention) {
            log::error!("Article purge failed: {}", e);
        }
    }

    commands::thumbnails::cleanup_thumbnail_cache(
        &state.paths.thumbnail_dir(),
        THUMBNAIL_CACHE_MAX_AGE_DAYS,
        THUMBNAIL_CACHE_MAX_BYTES,
    );
}

/// Show a startup failure before exiting. The window is hidden and logging may
/// not be configured yet, so a dialog is the only reliable channel.
fn fatal(app: &tauri::AppHandle, message: String) -> ! {
    log::error!("Startup failed: {}", message);
    eprintln!("[startup] {}", message);
    app.dialog()
        .message(&message)
        .title("FeedMee could not start")
        .blocking_show();
    std::process::exit(1);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            use log::info;

            let app_handle = app.handle().clone();
            let app_paths = paths::AppPaths::resolve(&app_handle)
                .unwrap_or_else(|e| fatal(&app_handle, format!("Could not find the app directories: {}", e)));
            if let Err(e) = app_paths.create_dirs() {
                fatal(&app_handle, format!("Could not create the app directories: {}", e));
            }

            let app_settings = settings::load_settings(&app_paths.config_dir);
            match startup::init_logging(&app_paths.logs_dir, &app_settings.log_level) {
                Ok(handle) => {
                    app.manage(handle);
                },
                Err(e) => eprintln!("[startup] logger init failed: {}", e),
            }

            info!("Starting FeedMee application");

            let db_path = app_paths.db_path();
            let conn = startup::setup_database(&db_path).unwrap_or_else(|e| fatal(&app_handle, e));

            let http_client = startup::build_http_client();

            app.manage(AppState {
                db: Mutex::new(conn),
                settings: Mutex::new(app_settings),
                http_client,
                og_image_semaphore: Arc::new(tokio::sync::Semaphore::new(OG_IMAGE_FETCH_CONCURRENCY)),
                paths: app_paths,
            });

            let Some(window) = app.get_webview_window("main") else {
                fatal(
                    &app_handle,
                    "The main window is missing from the app configuration.".to_string(),
                );
            };
            startup::setup_window(&window);
            startup::reveal_window_fallback(&window);

            let maintenance_handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(MAINTENANCE_START_DELAY_SECS));
                loop {
                    let state = maintenance_handle.state::<AppState>();
                    run_maintenance(&state);
                    if let Err(e) = commands::backup::run_auto_backup(&state.db, &state.paths.backup_dir()) {
                        log::error!("Auto-backup failed: {}", e);
                    }
                    std::thread::sleep(MAINTENANCE_INTERVAL);
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                // Never restore DECORATIONS (stale state from the old borderless
                // build would re-hide the native titlebar) or VISIBLE.
                .with_state_flags(StateFlags::all() - StateFlags::DECORATIONS - StateFlags::VISIBLE)
                .skip_initial_state("main")
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::show_main_window,
            commands::get_folders_with_feeds,
            commands::get_articles_for_feed,
            commands::get_articles_for_folder,
            commands::get_latest_articles,
            commands::get_saved_articles,
            commands::get_app_settings,
            commands::save_app_settings,
            commands::get_shortcuts,
            commands::save_shortcuts,
            commands::open_latest_log,
            commands::get_storage_stats,
            commands::clear_thumbnail_cache,
            commands::vacuum_database,
            commands::create_folder,
            commands::mark_article_saved,
            commands::mark_article_read,
            commands::mark_all_read,
            commands::import_opml,
            commands::export_opml,
            commands::refresh_feed,
            commands::add_feed,
            commands::rename_folder,
            commands::rename_feed,
            commands::delete_feed,
            commands::delete_folder,
            commands::move_feed,
            commands::get_article_content,
            commands::get_feed_unread_count,
            commands::search_articles,
            commands::pick_system_font,
            commands::get_tags_for_article,
            commands::get_all_tags,
            commands::add_tag,
            commands::remove_tag,
            commands::delete_tag,
            commands::get_thumbnail
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            panic!("error while running tauri application: {}", e);
        });
}
