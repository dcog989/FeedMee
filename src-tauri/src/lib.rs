pub mod commands;
pub mod db;
pub mod models;

#[allow(unused_imports)]
use log::{error, info, warn};
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    db: Mutex<rusqlite::Connection>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use simplelog::*;
    use std::fs::File;

    tauri::Builder::default()
        .setup(|app| {
            // Setup Logging to File in AppData
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to find app data dir");

            if !app_data_dir.exists() {
                std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
            }

            let log_path = app_data_dir.join("feedmee.log");

            // Initialize SimpleLogger (File + Term)
            let _ = CombinedLogger::init(vec![
                TermLogger::new(
                    LevelFilter::Info,
                    Config::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(
                    LevelFilter::Info,
                    Config::default(),
                    File::create(log_path).unwrap(),
                ),
            ]);

            info!("Starting FeedMee application");
            info!("App data directory: {:?}", app_data_dir);

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
            commands::mark_article_read,
            commands::mark_all_read, // Added
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
            // We can't log here easily if setup failed, but panic will print to stderr
            panic!("error while running tauri application: {}", e);
        });
}
