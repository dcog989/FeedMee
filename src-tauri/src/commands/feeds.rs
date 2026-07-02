use log::info;
use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn add_feed(
    url: String,
    folder_id: Option<i64>,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let source = crate::connectors::detect_feed(&url, &state.http_client).await?;
    let feed_id = source.add(folder_id, &state).await?;
    info!("add_feed: url={}, feed_id={}", url, feed_id);
    Ok(feed_id)
}
