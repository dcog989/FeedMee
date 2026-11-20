use crate::{
    AppState, db,
    settings::{self, AppSettings},
};
use log::info;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use tauri::{Manager, State};
use tauri_plugin_window_state::{StateFlags, WindowExt};

#[tauri::command]
pub fn show_main_window(app: tauri::AppHandle) {
    // Startup can fail before AppState is managed. Showing the window then only
    // produces failing calls, so leave it hidden behind the error dialog.
    if app.try_state::<AppState>().is_none() {
        return;
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        // Restore size/position/maximize after showing: applying them to a hidden
        // window (as the plugin's on-window-ready restore does) is dropped on GTK
        // and leaves the window oversized, clipping content top and bottom.
        // Mirror the plugin's flags: never restore DECORATIONS/VISIBLE.
        let _ = window.restore_state(StateFlags::all() - StateFlags::DECORATIONS - StateFlags::VISIBLE);
        let _ = window.set_focus();
    }
}

#[derive(Serialize)]
pub struct AppInfo {
    pub version: String,
    pub data_path: String,
    pub logs_path: String,
    pub backup_path: String,
    pub settings_path: String,
}

#[tauri::command]
pub fn get_app_info(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<AppInfo, String> {
    let version = app.package_info().version.to_string();

    Ok(AppInfo {
        version,
        data_path: state.paths.data_dir.to_string_lossy().to_string(),
        logs_path: state.paths.logs_dir.to_string_lossy().to_string(),
        backup_path: state.paths.backup_dir().to_string_lossy().to_string(),
        settings_path: state
            .paths
            .config_dir
            .join(settings::SETTINGS_FILENAME)
            .to_string_lossy()
            .to_string(),
    })
}

#[tauri::command]
pub fn get_app_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().unwrap();
    Ok(settings.clone())
}

#[tauri::command]
pub fn save_app_settings(
    new_settings: AppSettings,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut settings_guard = state.settings.lock().unwrap();
    let retention_changed = new_settings.article_retention_days != settings_guard.article_retention_days;
    let updated = new_settings;
    settings::save_settings(&state.paths.config_dir, &updated)?;
    if let Some(logger) = app.try_state::<flexi_logger::LoggerHandle>()
        && let Err(e) = logger.parse_new_spec(&crate::startup::log_spec(&updated.log_level))
    {
        log::error!("Failed to apply log level: {}", e);
    }
    *settings_guard = updated;
    drop(settings_guard);

    if retention_changed {
        // A lower retention must apply now, not at the next launch.
        let app = app.clone();
        let _ = tauri::async_runtime::spawn_blocking(move || {
            let state = app.state::<AppState>();
            let retention = match state.settings.lock() {
                Ok(settings) => settings.article_retention_days,
                Err(_) => return,
            };
            if let Ok(conn) = state.db.lock()
                && let Err(e) = db::purge_old_articles(&conn, retention)
            {
                log::error!("Article purge after a retention change failed: {}", e);
            }
        });
    }
    Ok(())
}

#[tauri::command]
pub fn pick_system_font(app_handle: tauri::AppHandle) -> Result<String, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app_handle
        .run_on_main_thread(move || {
            let result = pick_font_platform();
            let _ = tx.send(result);
        })
        .map_err(|e| format!("Failed to dispatch font picker: {}", e))?;

    rx.recv().map_err(|_| "Font picker failed".to_string())?
}

#[cfg(target_os = "linux")]
fn pick_font_platform() -> Result<String, String> {
    use gtk::prelude::*;

    let dialog = gtk::FontChooserDialog::new(Some("Select Font — FeedMee"), None::<&gtk::Window>);
    let res = dialog.run();

    let result = if res == gtk::ResponseType::Ok {
        let name = dialog
            .font_desc()
            .and_then(|desc| desc.family())
            .map(|family| family.trim().to_string())
            .unwrap_or_default();
        if name.is_empty() {
            Err("No font selected".to_string())
        } else {
            Ok(name)
        }
    } else {
        Err("Font selection cancelled".to_string())
    };

    dialog.close();
    result
}

#[cfg(not(target_os = "linux"))]
fn pick_font_platform() -> Result<String, String> {
    Err("Native font picker is not yet supported on this platform. You can type the font name directly in the input field.".to_string())
}

#[tauri::command]
pub fn get_shortcuts(state: State<'_, AppState>) -> Result<std::collections::HashMap<String, String>, String> {
    let shortcuts_path = state.paths.shortcuts_path();
    if !shortcuts_path.exists() {
        return Ok(std::collections::HashMap::new());
    }

    let content = fs::read_to_string(&shortcuts_path).map_err(|e| e.to_string())?;
    match serde_json::from_str::<std::collections::HashMap<String, String>>(&content) {
        Ok(shortcuts) => Ok(shortcuts),
        Err(e) => {
            log::error!("Failed to parse shortcuts file {}: {}", shortcuts_path.display(), e);
            let stamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let backup = PathBuf::from(format!("{}.corrupt-{}", shortcuts_path.display(), stamp));
            if let Err(rename_err) = fs::rename(&shortcuts_path, &backup) {
                log::error!("Failed to move unreadable shortcuts file aside: {}", rename_err);
            }
            Ok(std::collections::HashMap::new())
        },
    }
}

#[tauri::command]
pub fn save_shortcuts(
    shortcuts: std::collections::HashMap<String, String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let path = state.paths.shortcuts_path();
    fs::create_dir_all(&state.paths.config_dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&shortcuts).map_err(|e| e.to_string())?;
    crate::startup::write_atomic(&path, json.as_bytes()).map_err(|e| e.to_string())?;
    info!("Shortcuts saved to disk");
    Ok(())
}
