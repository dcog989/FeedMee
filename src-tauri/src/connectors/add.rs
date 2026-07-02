use log::info;

use crate::commands::scraper::backfill_og_images;
use crate::{AppState, db, models::Article};

pub(super) async fn add_detected_feed(
    source: super::DetectedFeed,
    folder_id: Option<i64>,
    state: &AppState,
) -> Result<i64, String> {
    let (title, feed_url, feed_type, mut articles) = resolve_source(source, state).await?;

    {
        let conn = state.db.lock().unwrap();
        if db::feed_exists_by_url(&conn, &feed_url).map_err(|e| e.to_string())? {
            return Err("Feed already exists".to_string());
        }
    }

    let feed_id = {
        let conn = state.db.lock().unwrap();
        db::create_feed(&conn, &title, &feed_url, folder_id, &feed_type)
            .map_err(|e| e.to_string())?
    };

    for a in &mut articles {
        a.feed_id = feed_id;
    }

    backfill_og_images(&state.http_client, &mut articles).await;

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

    info!("add_feed: url={}, feed_id={}", feed_url, feed_id);
    Ok(feed_id)
}

async fn resolve_source(
    source: super::DetectedFeed,
    state: &AppState,
) -> Result<(String, String, String, Vec<Article>), String> {
    match source {
        super::DetectedFeed::Bluesky { url } => {
            let (display_name, feed_url, articles) =
                super::bluesky::resolve_bluesky_source(&url, &state.http_client).await?;
            Ok((display_name, feed_url, "bluesky".to_string(), articles))
        },
        super::DetectedFeed::Rss { feed, url } => {
            let title = feed
                .title
                .as_ref()
                .map(|t| t.content.clone())
                .unwrap_or_else(|| "Untitled Feed".to_string());
            let articles = super::rss::entries_to_articles(feed.entries, 0, &url);
            Ok((title, url, "rss".to_string(), articles))
        },
        super::DetectedFeed::Website { url, content } => {
            let html = String::from_utf8_lossy(&content);
            let (title, articles) = super::website::extract_website_articles(&html, &url)?;
            Ok((title, url, "website".to_string(), articles))
        },
    }
}
