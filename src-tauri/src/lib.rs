pub mod commands;
pub mod connectors;
pub mod db;
pub mod models;
pub mod paths;
pub mod settings;
pub mod startup;

use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_window_state::StateFlags;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub settings: Mutex<settings::AppSettings>,
    pub http_client: reqwest::Client,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            use log::info;

            let (_local_dir, _config_dir, logs_dir, db_dir) = startup::create_dirs();
            startup::rotate_logs(&logs_dir);

            let mut app_settings = settings::load_settings();
            let log_level = startup::parse_log_level(&app_settings.log_level);
            startup::init_logging(&logs_dir, log_level);

            info!("Starting FeedMee application");

            let db_path = db_dir.join(db::DB_FILENAME);
            let conn = startup::setup_database(&db_path, &mut app_settings);

            let http_client = startup::build_http_client();

            app.manage(AppState {
                db: Mutex::new(conn),
                settings: Mutex::new(app_settings),
                http_client,
            });

            let window = app.get_webview_window("main").unwrap();
            startup::setup_window(&window);

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = app_handle.state::<AppState>();

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;

                let (do_vacuum, retention) = {
                    let s = state.settings.lock().unwrap();
                    (now - s.last_vacuum > 86400, s.article_retention_days)
                };

                let conn = state.db.lock().unwrap();

                if do_vacuum {
                    if let Err(e) = db::run_vacuum(&conn) {
                        log::error!("Maintenance VACUUM failed: {}", e);
                    } else {
                        let mut s = state.settings.lock().unwrap();
                        s.last_vacuum = now;
                        crate::settings::save_settings(&s);
                    }
                }

                if let Ok(count) = db::purge_old_articles(&conn, retention)
                    && count > 0
                {
                    log::info!("Startup: purged {} old articles", count);
                }

                drop(conn);

                let _ = commands::thumbnails::cleanup_thumbnail_cache(&app_handle, 7);
            });

            Ok(())
        })
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::all() - StateFlags::DECORATIONS)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::get_folders_with_feeds,
            commands::get_articles_for_feed,
            commands::get_articles_for_folder,
            commands::get_latest_articles,
            commands::get_saved_articles,
            commands::get_app_settings,
            commands::save_app_settings,
            commands::get_shortcuts,
            commands::save_shortcuts,
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
