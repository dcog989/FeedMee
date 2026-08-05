use async_trait::async_trait;
use scraper::{Html, Selector};

use crate::commands::scraper::{scrape_articles_from_page, scrape_og_image_from_html};
use crate::{AppState, db, models::Article};

use super::FeedConnector;

pub struct WebsiteConnector;

#[async_trait]
impl FeedConnector for WebsiteConnector {
    fn feed_type(&self) -> &'static str {
        "website"
    }

    async fn fetch_articles(
        &self,
        url: &str,
        state: &AppState,
    ) -> Result<(String, String, Vec<Article>), String> {
        let html = state
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        let (title, articles) = extract_website_articles(&html, url)?;
        Ok((title, url.to_string(), articles))
    }

    async fn refresh(&self, feed_url: &str, feed_id: i64, state: &AppState) -> Result<i64, String> {
        refresh_website_feed(feed_url, feed_id, state).await
    }
}

pub fn extract_website_articles(
    html: &str,
    page_url: &str,
) -> Result<(String, Vec<Article>), String> {
    let title = extract_page_title(html, page_url);
    let og_image = scrape_og_image_from_html(html, page_url);
    let mut articles = scrape_articles_from_page(html, page_url);
    for a in &mut articles {
        a.feed_id = 0;
        if a.image_url.is_empty() {
            a.image_url = og_image.clone().unwrap_or_default();
        }
    }
    if articles.is_empty() {
        return Err("No articles found on page".to_string());
    }
    Ok((title, articles))
}

pub fn extract_page_title(html: &str, fallback_url: &str) -> String {
    let document = Html::parse_document(html);
    let title_sel = Selector::parse("title").ok();
    title_sel
        .and_then(|sel| document.select(&sel).next())
        .map(|el| el.text().collect::<String>())
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| fallback_url.to_string())
}

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
    let mut conn = state.db.lock().unwrap();
    let count = db::batch_insert_articles(&mut conn, &articles).map_err(|e| e.to_string())?;
    let _ = db::update_feed_error(&conn, feed_id, false);
    Ok(count)
}

async fn refresh_website_feed(
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
