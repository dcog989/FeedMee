use crate::{AppState, db, settings::{self, AppSettings}};
use log::info;
use serde::Serialize;
use std::fs;
use tauri::{AppHandle, Manager, State};

#[derive(Serialize)]
pub struct AppInfo {
    pub version: String,
    pub data_path: String,
    pub logs_path: String,
    pub db_path: String,
}

#[tauri::command]
pub fn get_app_info(app: tauri::AppHandle) -> Result<AppInfo, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    let version = app.package_info().version.to_string();

    Ok(AppInfo {
        version,
        data_path: app_data_dir.to_string_lossy().to_string(),
        logs_path: app_data_dir.join("Logs").to_string_lossy().to_string(),
        db_path: app_data_dir
            .join("Database")
            .join(db::DB_FILENAME)
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
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut settings_guard = state.settings.lock().unwrap();
    *settings_guard = new_settings.clone();

    if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        settings::save_settings(&app_data_dir, &new_settings);
        Ok(())
    } else {
        Err("Could not determine app data directory".to_string())
    }
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
        match dialog.font() {
            Some(font_desc) => {
                let name = font_desc
                    .rsplitn(2, ' ')
                    .last()
                    .unwrap_or(&font_desc)
                    .trim()
                    .to_string();
                if name.is_empty() {
                    Err("No font selected".to_string())
                } else {
                    Ok(name)
                }
            }
            None => Err("No font selected".to_string()),
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
pub fn get_shortcuts(
    app: tauri::AppHandle,
) -> Result<std::collections::HashMap<String, String>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let shortcuts_path = app_data_dir.join("shortcuts.json");

    if shortcuts_path.exists() {
        let content = fs::read_to_string(&shortcuts_path).map_err(|e| e.to_string())?;
        let shortcuts: std::collections::HashMap<String, String> =
            serde_json::from_str(&content).unwrap_or_default();
        Ok(shortcuts)
    } else {
        Ok(std::collections::HashMap::new())
    }
}

#[tauri::command]
pub fn save_shortcuts(
    shortcuts: std::collections::HashMap<String, String>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let shortcuts_path = app_data_dir.join("shortcuts.json");

    let json = serde_json::to_string_pretty(&shortcuts).map_err(|e| e.to_string())?;
    fs::write(shortcuts_path, json).map_err(|e| e.to_string())?;

    info!("Shortcuts saved to disk");
    Ok(())
}
