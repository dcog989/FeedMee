use crate::{AppState, db, models::Article};

pub(crate) async fn add_feed_with_articles(
    title: &str,
    feed_url: &str,
    feed_type: &str,
    mut articles: Vec<Article>,
    cursor: Option<String>,
    folder_id: Option<i64>,
    state: &AppState,
) -> Result<i64, String> {
    let feed_id = {
        let conn = state.db.lock().unwrap();
        if db::feed_exists_by_url(&conn, feed_url, None).map_err(|e| e.to_string())? {
            return Err("Feed already exists".to_string());
        }
        db::create_feed(&conn, title, feed_url, folder_id, feed_type).map_err(|e| e.to_string())?
    };

    for a in &mut articles {
        a.feed_id = feed_id;
    }

    let cutoff = state.article_retention_cutoff();
    let count = {
        let mut conn = state.db.lock().unwrap();
        match db::batch_insert_articles(&mut conn, &articles, cutoff) {
            Ok(count) => count,
            Err(e) => {
                let _ = db::delete_feed(&conn, feed_id);
                return Err(e.to_string());
            },
        }
    };

    if count == 0 && feed_type == "website" {
        let conn = state.db.lock().unwrap();
        let _ = db::delete_feed(&conn, feed_id);
        return Err("No articles found on page".to_string());
    }

    if let Some(cursor) = cursor {
        let conn = state.db.lock().unwrap();
        let _ = db::set_bluesky_cursor(&conn, feed_id, &cursor);
    }

    Ok(feed_id)
}
