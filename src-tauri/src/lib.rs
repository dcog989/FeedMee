pub mod commands;
pub mod db;
pub mod models;

use log::{error, info, warn};
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    db: Mutex<rusqlite::Connection>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize the logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info) // Default to Info level
        .init();

    info!("Starting FeedMee application");

    tauri::Builder::default()
        .setup(|app| {
            info!("Setting up application");

            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to find app data dir");

            info!("App data directory: {:?}", app_data_dir);

            if !app_data_dir.exists() {
                info!("Creating app data directory");
                std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
            }

            let db_path = app_data_dir.join("feedmee.sqlite");
            info!("Opening database at: {:?}", db_path);

            let mut conn = rusqlite::Connection::open(&db_path).map_err(|e| {
                error!("Failed to open database: {}", e);
                format!("Database open failed: {}", e)
            })?;

            info!("Initializing database schema");
            if let Err(e) = db::init_db(&mut conn) {
                error!("Schema initialization failed: {}", e);
                panic!("Schema init failed: {}", e);
            }

            info!("Database initialized successfully");

            app.manage(AppState {
                db: Mutex::new(conn),
            });

            info!("Application setup complete");
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
            commands::create_folder,
            commands::mark_article_saved,
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
            error!("Error while running Tauri application: {}", e);
            panic!("error while running tauri application: {}", e);
        });

    info!("FeedMee application stopped");
}
