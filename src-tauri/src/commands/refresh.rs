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
    let bytes = html.as_bytes();
    let mut i = 0;
    let mut count = 0i32;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            if i + 2 < bytes.len() && bytes[i + 1] == b'p' {
                let next = bytes[i + 2];
                if next == b'>' || next == b' ' {
                    count += 1;
                }
            } else if i + 3 < bytes.len() && bytes[i + 1] == b'b' && bytes[i + 2] == b'r' {
                let next = bytes[i + 3];
                if next == b'>' || next == b' ' || next == b'/' {
                    count += 1;
                }
            }
        }
        i += 1;
    }
    count >= 2
}

fn extract_with_css_selectors(html: &str) -> Option<String> {
    let selectors = [
        "[role=\"main\"]",
        "article.full",
        "article[class*=\"full\"]",
        "article",
        "#main-content",
        "#content",
        ".content",
        "[itemprop=\"articleBody\"]",
        ".post-content",
        ".entry-content",
        ".article-body",
        ".story-body",
        ".RichTextContainer",
        "[data-component=\"text-block\"]",
    ];

    let document = Html::parse_document(html);
    for sel_str in &selectors {
        let selector = match Selector::parse(sel_str) {
            Ok(s) => s,
            Err(_) => continue,
        };
        if let Some(el) = document.select(&selector).next() {
            let inner = el.inner_html();
            let text_len = content_text_len(&inner);
            if text_len > 100 && has_paragraph_structure(&inner) {
                debug!(
                    "extract_with_css: matched '{}' ({} chars)",
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
    let html = state
        .http_client
        .get(&url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let readability_ok = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Ok(readability) =
            Readability::new(&html, Some(&url), Some(ReadabilityOptions::default()))
            && let Some(article) = readability.parse()
            && let Some(content) = article.content
        {
            if content_text_len(&content) > 100 && has_paragraph_structure(&content) {
                debug!(
                    "get_article_content: readabilityrs extracted {} chars",
                    content.len()
                );
                return Some(content);
            }
            debug!(
                "get_article_content: readabilityrs content too short or no paragraphs ({} chars), falling back to CSS",
                content.len()
            );
        }
        None
    }));

    if let Ok(Some(content)) = readability_ok {
        return Ok(content);
    }

    if readability_ok.is_err() {
        debug!("get_article_content: readabilityrs panicked, falling back to CSS");
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
