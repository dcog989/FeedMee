use log::debug;
use readabilityrs::{Readability, ReadabilityOptions};
use scraper::{Html, Selector};
use tauri::State;

use crate::{AppState, db};

fn content_text_len(html: &str) -> usize {
    let doc = Html::parse_fragment(html);
    doc.root_element().text().collect::<String>().trim().len()
}

fn has_paragraph_structure(html: &str) -> bool {
    let breaks = html.matches("<p>").count() + html.matches("<br").count();
    breaks >= 2
}

fn extract_with_css_selectors(html: &str) -> Option<String> {
    let selectors = [
        "article.full",
        "article[class*=\"full\"]",
        "article",
        "#content",
        ".content",
        "[itemprop=\"articleBody\"]",
        ".post-content",
        ".entry-content",
        ".article-body",
        ".story-body",
    ];

    let document = Html::parse_document(html);
    for sel_str in &selectors {
        let selector = Selector::parse(sel_str).ok()?;
        if let Some(el) = document.select(&selector).next() {
            let inner = el.inner_html();
            let text_len = content_text_len(&inner);
            if text_len > 200 && has_paragraph_structure(&inner) {
                debug!(
                    "extract_with_css: matched selector '{}' ({} chars)",
                    sel_str,
                    inner.len()
                );
                return Some(inner);
            }
        }
    }
    None
}

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

    if let Ok(readability) =
        Readability::new(&html, Some(&url), Some(ReadabilityOptions::default()))
    {
        if let Some(article) = readability.parse() {
            if let Some(content) = article.content {
                if content_text_len(&content) > 200 && has_paragraph_structure(&content) {
                    debug!(
                        "get_article_content: readabilityrs extracted {} chars",
                        content.len()
                    );
                    return Ok(content);
                }
                debug!(
                    "get_article_content: readabilityrs content too short or no paragraphs ({} chars), falling back to CSS",
                    content.len()
                );
            }
        }
    }

    extract_with_css_selectors(&html).ok_or_else(|| "No content extracted".to_string())
}

#[tauri::command]
pub async fn refresh_feed(feed_id: i64, state: State<'_, AppState>) -> Result<i64, String> {
    let (url, feed_type) = {
        let conn = state.db.lock().unwrap();
        let feed = db::get_feed(&conn, feed_id).map_err(|e| e.to_string())?;
        let _ = db::update_feed_error(&conn, feed_id, false);
        (feed.url, feed.feed_type)
    };

    crate::connectors::registry()
        .refresh(&feed_type, &url, feed_id, &state)
        .await
}
