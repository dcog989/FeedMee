use tauri::State;

use crate::{AppState, db};

#[tauri::command]
pub async fn refresh_feed(feed_id: i64, state: State<'_, AppState>) -> Result<i64, String> {
    let (url, feed_type) = {
        let conn = state.db.lock().unwrap();
        let feed = db::get_feed(&conn, feed_id).map_err(|e| e.to_string())?;
        (feed.url, feed.feed_type)
    };

    let result = crate::connectors::registry()
        .refresh(&feed_type, &url, feed_id, &state)
        .await;

    {
        let conn = state.db.lock().unwrap();
        let recorded = match &result {
            Ok(_) => db::record_refresh_success(&conn, feed_id),
            Err(_) => db::record_refresh_failure(&conn, feed_id),
        };
        if let Err(e) = recorded {
            log::error!("Failed to record refresh outcome for feed {}: {}", feed_id, e);
        }
    }

    result
}
