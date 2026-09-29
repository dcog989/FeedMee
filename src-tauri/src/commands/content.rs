use dom_smoothie::{Config, Readability};
use log::debug;
use scraper::{Html, Selector};
use tauri::State;

use crate::AppState;

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
                debug!("extract_with_css: matched '{}' ({} chars)", sel_str, inner.len());
                return Some(inner);
            }
        }
    }
    None
}

fn extract_with_readability(html: &str, url: &str) -> Option<String> {
    let mut readability = Readability::new(html, Some(url), Some(Config::default())).ok()?;
    let article = readability.parse().ok()?;
    let content = article.content.to_string();

    if content_text_len(&content) > 100 && has_paragraph_structure(&content) {
        debug!("get_article_content: dom_smoothie extracted {} chars", content.len());
        return Some(content);
    }

    debug!(
        "get_article_content: dom_smoothie content too short or no paragraphs ({} chars), falling back to CSS",
        content.len()
    );
    None
}

#[tauri::command]
pub async fn get_article_content(url: String, state: State<'_, AppState>) -> Result<String, String> {
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

    if let Some(content) = extract_with_readability(&html, &url) {
        return Ok(content);
    }

    extract_with_css_selectors(&html).ok_or_else(|| "No content extracted".to_string())
}

#[cfg(test)]
mod tests {
    use super::{extract_with_css_selectors, extract_with_readability};

    fn article_html() -> String {
        let body = "<p>FeedMee is a desktop news feed reader that fetches RSS and Atom feeds in \
            the background and renders every article in a clean, distraction-free layout.</p>\
            <p>The reader keeps memory usage low by paginating results and caching thumbnails, \
            so even feeds with thousands of entries stay responsive while you scroll.</p>";
        format!(
            "<html><head><title>FeedMee review</title></head>\
             <body><nav>Home About Contact</nav>\
             <article class=\"post-content\">{body}</article>\
             <footer>Copyright</footer></body></html>"
        )
    }

    #[test]
    fn readability_extracts_article_body() {
        let html = article_html();
        let content = extract_with_readability(&html, "https://example.com/post")
            .expect("readability should extract the article body");
        assert!(content.contains("FeedMee is a desktop news feed reader"));
        assert!(content.contains("paginating results"));
    }

    #[test]
    fn css_fallback_extracts_known_container() {
        let html = article_html();
        let content = extract_with_css_selectors(&html).expect("css selector should match .post-content");
        assert!(content.contains("renders every article"));
    }

    #[test]
    fn css_fallback_ignores_short_fragments() {
        let html = "<html><body><div class=\"content\">too short</div></body></html>";
        assert!(extract_with_css_selectors(html).is_none());
    }
}
