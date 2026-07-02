use readabilityrs::{Readability, ReadabilityOptions};
use tauri::State;

use crate::{AppState, db};

#[tauri::command]
pub async fn get_article_content(
    url: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let client = state.http_client.clone();
    let html = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let options = ReadabilityOptions::default();
    let readability =
        Readability::new(&html, Some(&url), Some(options)).map_err(|e| format!("{:?}", e))?;
    let article = readability.parse().ok_or("Failed to parse content")?;
    article.content.ok_or("No content extracted".to_string())
}

#[tauri::command]
pub async fn refresh_feed(feed_id: i64, state: State<'_, AppState>) -> Result<i64, String> {
    let (url, feed_type) = {
        let conn = state.db.lock().unwrap();
        let feed = db::get_feed(&conn, feed_id).map_err(|e| e.to_string())?;
        let _ = db::update_feed_error(&conn, feed_id, false);
        (feed.url, feed.feed_type)
    };

    crate::connectors::refresh_feed_by_type(&feed_type, &url, feed_id, &state).await
}
