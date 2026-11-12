use async_trait::async_trait;
use scraper::{Html, Selector};

use crate::commands::scraper::{scrape_articles_from_page, scrape_og_image_from_html};
use crate::{AppState, db, models::Article};

use super::{FeedConnector, FetchedFeed, FetchedPage};

pub struct WebsiteConnector;

#[async_trait]
impl FeedConnector for WebsiteConnector {
    fn feed_type(&self) -> &'static str {
        "website"
    }

    async fn fetch_articles(
        &self,
        _url: &str,
        page: Option<&FetchedPage>,
        _state: &AppState,
    ) -> Result<FetchedFeed, String> {
        let page = page.ok_or("Website connector requires a prefetched page")?;
        let html = decode_html(&page.bytes, page.charset.as_deref());
        let base_url = page.final_url.to_string();

        let (title, articles) = extract_website_articles(&html, &base_url)?;
        Ok(FetchedFeed {
            title,
            url: base_url,
            articles,
        })
    }

    async fn refresh(&self, feed_url: &str, feed_id: i64, state: &AppState) -> Result<i64, String> {
        refresh_website_feed(feed_url, feed_id, state).await
    }
}

fn decode_html(bytes: &[u8], charset: Option<&str>) -> String {
    let encoding = charset
        .and_then(|label| encoding_rs::Encoding::for_label(label.as_bytes()))
        .unwrap_or(encoding_rs::UTF_8);
    encoding.decode(bytes).0.into_owned()
}

fn scrape_page(html: &str, page_url: &str, feed_id: i64) -> Vec<Article> {
    let og_image = scrape_og_image_from_html(html, page_url);
    let mut articles = scrape_articles_from_page(html, page_url);
    for a in &mut articles {
        a.feed_id = feed_id;
        if a.image_url.is_empty() {
            a.image_url = og_image.clone().unwrap_or_default();
        }
    }
    articles
}

pub fn extract_website_articles(html: &str, page_url: &str) -> Result<(String, Vec<Article>), String> {
    let articles = scrape_page(html, page_url, 0);
    if articles.is_empty() {
        return Err("No articles found on page".to_string());
    }
    Ok((extract_page_title(html, page_url), articles))
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

fn scrape_and_insert(html: &str, page_url: &str, feed_id: i64, state: &AppState) -> Result<usize, String> {
    let articles = scrape_page(html, page_url, feed_id);
    let cutoff = state.article_retention_cutoff();
    let mut conn = state.db.lock().unwrap();
    db::batch_insert_articles(&mut conn, &articles, cutoff).map_err(|e| e.to_string())
}

async fn refresh_website_feed(feed_url: &str, feed_id: i64, state: &AppState) -> Result<i64, String> {
    let client = state.http_client.clone();
    let response = client.get(feed_url).send().await.map_err(|e| e.to_string())?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP {} for {}", status, feed_url));
    }
    let charset = super::response_charset(&response);
    let bytes = super::read_body(response).await?;
    let html = decode_html(&bytes, charset.as_deref());

    let _ = scrape_and_insert(&html, feed_url, feed_id, state)?;

    let conn = state.db.lock().unwrap();
    Ok(db::get_feed_unread_count(&conn, feed_id).unwrap_or(0))
}
