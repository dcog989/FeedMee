use crate::commands::scraper::{scrape_articles_from_page, scrape_og_image_from_html};
use crate::{AppState, db};
use log::debug;
use scraper::{Html, Selector};

async fn scrape_and_insert(
    html: &str,
    page_url: &str,
    feed_id: i64,
    state: &AppState,
) -> Result<usize, String> {
    let og_image = scrape_og_image_from_html(html, page_url);
    let mut articles = scrape_articles_from_page(html, page_url);
    for a in &mut articles {
        a.feed_id = feed_id;
        if a.image_url.is_empty() {
            a.image_url = og_image.clone().unwrap_or_default();
        }
    }
    let conn = state.db.lock().unwrap();
    let count = db::batch_insert_articles(&conn, &articles).map_err(|e| e.to_string())?;
    let _ = db::update_feed_error(&conn, feed_id, false);
    Ok(count)
}

pub async fn add_website_feed(
    url: &str,
    content_bytes: &[u8],
    folder_id: Option<i64>,
    state: &AppState,
) -> Result<i64, String> {
    {
        let conn = state.db.lock().unwrap();
        if db::feed_exists_by_url(&conn, url).map_err(|e| e.to_string())? {
            return Err("Feed already exists".to_string());
        }
    }

    let html = String::from_utf8_lossy(content_bytes);
    let title = {
        let document = Html::parse_document(&html);
        let title_sel = Selector::parse("title").ok();
        title_sel
            .and_then(|sel| document.select(&sel).next())
            .map(|el| el.text().collect::<String>())
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| url.to_string())
    };

    let feed_id = {
        let conn = state.db.lock().unwrap();
        db::create_feed(&conn, &title, url, folder_id, "website").map_err(|e| e.to_string())?
    };

    let count = scrape_and_insert(&html, url, feed_id, state).await?;

    if count == 0 {
        if let Ok(conn) = state.db.lock() {
            let _ = db::delete_feed(&conn, feed_id);
        }
        return Err(format!("No articles found on page: {}", url));
    }

    debug!("add_website_feed: feed_id={}, articles={}", feed_id, count);
    Ok(feed_id)
}

pub async fn refresh_website_feed(
    feed_url: &str,
    feed_id: i64,
    state: &AppState,
) -> Result<i64, String> {
    let client = state.http_client.clone();
    let response = client.get(feed_url).send().await.map_err(|e| {
        let conn = state.db.lock().unwrap();
        let _ = db::update_feed_error(&conn, feed_id, true);
        e.to_string()
    })?;
    let html = response.text().await.map_err(|e| {
        let conn = state.db.lock().unwrap();
        let _ = db::update_feed_error(&conn, feed_id, true);
        e.to_string()
    })?;

    let _ = scrape_and_insert(&html, feed_url, feed_id, state).await?;

    let conn = state.db.lock().unwrap();
    let _ = db::update_feed_error(&conn, feed_id, false);
    Ok(db::get_feed_unread_count(&conn, feed_id).unwrap_or(0))
}
