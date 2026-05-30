pub mod commands;
pub mod connectors;
pub mod db;
pub mod models;
pub mod settings;

#[allow(unused_imports)]
use log::{error, info, warn};
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_window_state::StateFlags;

#[cfg(target_os = "linux")]
use gtk::prelude::GtkWindowExt;

pub struct AppState {
    db: Mutex<rusqlite::Connection>,
    settings: Mutex<settings::AppSettings>,
    pub http_client: reqwest::Client,
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

            let logs_dir = app_data_dir.join("Logs");
            let db_dir = app_data_dir.join("Database");

            for dir in [&app_data_dir, &logs_dir, &db_dir] {
                if !dir.exists() {
                    std::fs::create_dir_all(dir).expect("failed to create app data dir");
                }
            }

            // --- Log Rotation ---
            let log_file_name = "feedmee.log";
            let max_logs = 5;

            // Delete oldest if exists
            let oldest_log = logs_dir.join(format!("feedmee.{}.log", max_logs));
            if oldest_log.exists() {
                let _ = std::fs::remove_file(oldest_log);
            }

            // Shift existing logs: 4->5, 3->4, etc.
            for i in (1..max_logs).rev() {
                let current = logs_dir.join(format!("feedmee.{}.log", i));
                let next = logs_dir.join(format!("feedmee.{}.log", i + 1));
                if current.exists() {
                    let _ = std::fs::rename(current, next);
                }
            }

            // Shift main log to .1
            let current_log = logs_dir.join(log_file_name);
            if current_log.exists() {
                let _ = std::fs::rename(&current_log, logs_dir.join("feedmee.1.log"));
            }
            // ---------------------

            // Load Settings
            let mut app_settings = settings::load_settings(&app_data_dir);

            let log_level = match app_settings.log_level.to_lowercase().as_str() {
                "error" => LevelFilter::Error,
                "warn" => LevelFilter::Warn,
                "debug" => LevelFilter::Debug,
                "trace" => LevelFilter::Trace,
                _ => LevelFilter::Info,
            };

            let log_path = logs_dir.join(log_file_name);

            let log_config = ConfigBuilder::new()
                .add_filter_ignore_str("html5ever")
                .add_filter_ignore_str("selectors")
                .add_filter_ignore_str("scraper")
                .add_filter_ignore_str("tendril")
                .set_time_format_rfc3339()
                .build();

            let _ = CombinedLogger::init(vec![
                TermLogger::new(
                    log_level,
                    log_config.clone(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(log_level, log_config, File::create(log_path).unwrap()),
            ]);

            info!("Starting FeedMee application");
            info!("Settings loaded: {:?}", app_settings);

            let db_path = db_dir.join(db::DB_FILENAME);

            let mut conn = rusqlite::Connection::open(&db_path).map_err(|e| {
                error!("Failed to open database: {}", e);
                format!("Database open failed: {}", e)
            })?;

            if let Err(e) = db::init_db(&mut conn) {
                error!("Schema initialization failed: {}", e);
                panic!("Schema init failed: {}", e);
            }

            // Check Vacuum (every 24 hours = 86400 seconds)
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            if now - app_settings.last_vacuum > 86400 {
                if let Err(e) = db::run_vacuum(&conn) {
                    error!("Maintenance VACUUM failed: {}", e);
                } else {
                    app_settings.last_vacuum = now;
                    settings::save_settings(&app_data_dir, &app_settings);
                }
            }

            // Purge old articles on startup
            if let Ok(count) = db::purge_old_articles(&conn, app_settings.article_retention_days)
                && count > 0
            {
                info!("Startup: purged {} old articles", count);
            }

            // Clean up stale thumbnails on startup (older than 7 days)
            let _ = commands::thumbnails::cleanup_thumbnail_cache(app.handle(), 7);

            let http_client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("failed to build HTTP client");

            app.manage(AppState {
                db: Mutex::new(conn),
                settings: Mutex::new(app_settings),
                http_client,
            });

            // Background maintenance thread (runs every 6 hours)
            let maint_handle = app.handle().clone();
            std::thread::spawn(move || {
                let six_hours = std::time::Duration::from_secs(6 * 3600);
                loop {
                    std::thread::sleep(six_hours);
                    let state = maint_handle.state::<AppState>();

                    // Purge old articles
                    let retention = {
                        let settings = state.settings.lock().unwrap();
                        settings.article_retention_days
                    };
                    if let Ok(count) = db::purge_old_articles(&state.db.lock().unwrap(), retention)
                        && count > 0
                    {
                        info!("Maintenance: purged {} old articles", count);
                    }

                    // Vacuum check
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    let needs_vacuum = {
                        let settings = state.settings.lock().unwrap();
                        now - settings.last_vacuum > 86400
                    };
                    if needs_vacuum {
                        if let Err(e) = db::run_vacuum(&state.db.lock().unwrap()) {
                            error!("Maintenance VACUUM failed: {}", e);
                        } else {
                            if let Ok(app_data_dir) = maint_handle.path().app_data_dir() {
                                let mut settings = state.settings.lock().unwrap();
                                settings.last_vacuum = now;
                                settings::save_settings(&app_data_dir, &settings);
                            }
                        }
                    }

                    // Clean up stale thumbnails
                    let _ = commands::thumbnails::cleanup_thumbnail_cache(&maint_handle, 7);
                }
            });

            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "linux")]
            {
                if let Ok(gtk_window) = window.gtk_window() {
                    gtk_window.set_titlebar(None::<&gtk::Widget>);

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
            {
                match window.set_icon(tauri::include_image!("icons/32x32.png")) {
                    Ok(_) => info!("Window icon set successfully"),
                    Err(e) => warn!("Failed to set window icon: {}", e),
                }
            }

            Ok(())
        })
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_window_state::Builder::default().with_state_flags(StateFlags::all() - StateFlags::DECORATIONS).build())
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
