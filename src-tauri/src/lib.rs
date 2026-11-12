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
use tauri_plugin_window_state::StateFlags;

const HTTP_FETCH_CONCURRENCY: usize = 8;
/// First maintenance pass waits so it never competes with the first paint.
const MAINTENANCE_START_DELAY_SECS: u64 = 200;
const MAINTENANCE_INTERVAL: Duration = Duration::from_secs(86400);
const VACUUM_INTERVAL_SECS: i64 = 86400;
const THUMBNAIL_CACHE_MAX_AGE_DAYS: u64 = 7;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub settings: Mutex<settings::AppSettings>,
    pub http_client: reqwest::Client,
    pub http_semaphore: Arc<tokio::sync::Semaphore>,
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

/// Purge expired articles, VACUUM at most once a day, and drop stale cached
/// thumbnails. Runs off the UI thread so the DB lock is never held during a
/// paint.
fn run_maintenance(state: &AppState) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let (do_vacuum, retention) = {
        let s = state.settings.lock().unwrap();
        (now - s.last_vacuum > VACUUM_INTERVAL_SECS, s.article_retention_days)
    };

    let vacuum_ok = {
        let conn = state.db.lock().unwrap();

        if let Err(e) = db::purge_old_articles(&conn, retention) {
            log::error!("Article purge failed: {}", e);
        }

        if do_vacuum {
            match db::run_vacuum(&conn) {
                Ok(()) => true,
                Err(e) => {
                    log::error!("Maintenance VACUUM failed: {}", e);
                    false
                },
            }
        } else {
            false
        }
    };

    if vacuum_ok {
        let mut s = state.settings.lock().unwrap();
        s.last_vacuum = now;
        if let Err(e) = crate::settings::save_settings(&state.paths.config_dir, &s) {
            log::error!("Failed to save maintenance settings: {}", e);
        }
    }

    commands::thumbnails::cleanup_thumbnail_cache(&state.paths.thumbnail_dir(), THUMBNAIL_CACHE_MAX_AGE_DAYS);
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
            let app_paths = paths::AppPaths::resolve(&app_handle).expect("failed to resolve app directories");
            app_paths.create_dirs().expect("failed to create app directories");

            let app_settings = settings::load_settings(&app_paths.config_dir);
            match startup::init_logging(&app_paths.logs_dir, &app_settings.log_level) {
                Ok(handle) => {
                    app.manage(handle);
                },
                Err(e) => eprintln!("[startup] logger init failed: {}", e),
            }

            info!("Starting FeedMee application");

            let db_path = app_paths.db_path();
            let conn = startup::setup_database(&db_path);

            let http_client = startup::build_http_client();

            app.manage(AppState {
                db: Mutex::new(conn),
                settings: Mutex::new(app_settings),
                http_client,
                http_semaphore: Arc::new(tokio::sync::Semaphore::new(HTTP_FETCH_CONCURRENCY)),
                paths: app_paths,
            });

            let window = app.get_webview_window("main").unwrap();
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
            commands::write_file,
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
