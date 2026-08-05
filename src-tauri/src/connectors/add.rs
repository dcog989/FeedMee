use crate::commands::scraper::backfill_og_images;
use crate::{AppState, db, models::Article};

pub(crate) async fn add_feed_with_articles(
    title: &str,
    feed_url: &str,
    feed_type: &str,
    mut articles: Vec<Article>,
    folder_id: Option<i64>,
    state: &AppState,
) -> Result<i64, String> {
    {
        let conn = state.db.lock().unwrap();
        if db::feed_exists_by_url(&conn, feed_url).map_err(|e| e.to_string())? {
            return Err("Feed already exists".to_string());
        }
    }

    let feed_id = {
        let conn = state.db.lock().unwrap();
        db::create_feed(&conn, title, feed_url, folder_id, feed_type).map_err(|e| e.to_string())?
    };

    for a in &mut articles {
        a.feed_id = feed_id;
    }

    backfill_og_images(&state.http_client, &mut articles, |_| true).await;

    {
        let conn = state.db.lock().unwrap();
        let count = db::batch_insert_articles(&conn, &articles).map_err(|e| e.to_string())?;
        let _ = db::update_feed_error(&conn, feed_id, false);
        if count == 0 && feed_type == "website" {
            drop(conn);
            if let Ok(conn) = state.db.lock() {
                let _ = db::delete_feed(&conn, feed_id);
            }
            return Err("No articles found on page".to_string());
        }
    }

    Ok(feed_id)
}
