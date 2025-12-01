pub mod commands;
pub mod db;
pub mod models;
pub mod settings;

#[allow(unused_imports)]
use log::{error, info, warn};
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    db: Mutex<rusqlite::Connection>,
    settings: Mutex<settings::AppSettings>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use simplelog::*;
    use std::fs::File;

    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to find app data dir");

            if !app_data_dir.exists() {
                std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
            }

            // Load Settings first to get log level
            let app_settings = settings::load_settings(&app_data_dir);

            let log_level = match app_settings.log_level.to_lowercase().as_str() {
                "error" => LevelFilter::Error,
                "warn" => LevelFilter::Warn,
                "debug" => LevelFilter::Debug,
                "trace" => LevelFilter::Trace,
                _ => LevelFilter::Info,
            };

            let log_path = app_data_dir.join("feedmee.log");

            // Initialize Logger
            let _ = CombinedLogger::init(vec![
                TermLogger::new(
                    log_level,
                    Config::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(
                    log_level,
                    Config::default(),
                    File::create(log_path).unwrap(),
                ),
            ]);

            info!("Starting FeedMee application");
            info!("Settings loaded: {:?}", app_settings);

            let db_path = app_data_dir.join("feedmee.sqlite");

            let mut conn = rusqlite::Connection::open(&db_path).map_err(|e| {
                error!("Failed to open database: {}", e);
                format!("Database open failed: {}", e)
            })?;

            if let Err(e) = db::init_db(&mut conn) {
                error!("Schema initialization failed: {}", e);
                panic!("Schema init failed: {}", e);
            }

            app.manage(AppState {
                db: Mutex::new(conn),
                settings: Mutex::new(app_settings),
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::get_folders_with_feeds,
            commands::get_articles_for_feed,
            commands::get_articles_for_folder,
            commands::get_latest_articles,
            commands::get_saved_articles,
            commands::get_app_settings, // Added
            commands::create_folder,
            commands::mark_article_saved,
            commands::mark_article_read,
            commands::mark_all_read,
            commands::import_opml,
            commands::export_opml,
            commands::write_file,
            commands::refresh_feed,
            commands::refresh_all_feeds,
            commands::add_feed,
            commands::rename_folder,
            commands::delete_feed,
            commands::delete_folder,
            commands::move_feed,
            commands::get_article_content
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            panic!("error while running tauri application: {}", e);
        });
}
