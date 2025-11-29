pub mod commands;
pub mod db;
pub mod models;

use std::sync::Mutex;

pub struct AppState {
    db: Mutex<rusqlite::Connection>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to find app data dir");
            if !app_data_dir.exists() {
                std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
            }

            let db_path = app_data_dir.join("feedmee.sqlite");
            let mut conn = rusqlite::Connection::open(&db_path).expect("Database open failed");

            if let Err(e) = db::init_db(&mut conn) {
                panic!("Schema init failed: {}", e);
            }

            app.manage(AppState {
                db: Mutex::new(conn),
            });

            // --- Intelligent Window Sizing ---
            use tauri::Manager;
            let window = app.get_webview_window("main").unwrap();

            // We defer this slightly to ensure the monitor is ready,
            // but in setup() it's usually fine.
            if let Ok(Some(monitor)) = window.current_monitor() {
                let screen_size = monitor.size();
                let width = (screen_size.width as f64 * 0.6).round() as u32;
                let height = (screen_size.height as f64 * 0.6).round() as u32;

                // Only set if we haven't restored a state (simplified check)
                // ideally the window-state plugin handles this, but for "First Run 60%" specific logic:
                let _ =
                    window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }));
                let _ = window.center();
            }

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::get_folders_with_feeds,
            commands::get_articles_for_feed,
            commands::import_opml,
            commands::refresh_feed,
            commands::add_feed,
            commands::rename_folder,
            commands::delete_feed,
            commands::delete_folder,
            commands::move_feed
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
