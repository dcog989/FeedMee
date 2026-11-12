use dom_smoothie::{Config, Readability};
use log::debug;
use scraper::{Html, Selector};
use tauri::State;

use crate::AppState;

/// Text below this is site chrome, not an article body.
const MIN_CONTENT_CHARS: usize = 100;

fn content_text_len(html: &str) -> usize {
    let doc = Html::parse_fragment(html);
    doc.root_element().text().collect::<String>().trim().len()
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
            if content_text_len(&inner) > MIN_CONTENT_CHARS {
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

    if content_text_len(&content) > MIN_CONTENT_CHARS {
        debug!("get_article_content: dom_smoothie extracted {} chars", content.len());
        return Some(content);
    }

    debug!(
        "get_article_content: dom_smoothie content too short ({} chars), falling back to CSS",
        content.len()
    );
    None
}

fn extract_content(html: &str, url: &str) -> Option<String> {
    extract_with_readability(html, url).or_else(|| extract_with_css_selectors(html))
}

#[tauri::command]
pub async fn get_article_content(url: String, state: State<'_, AppState>) -> Result<String, String> {
    if !crate::net::is_public_url(&url) {
        return Err("Article URL is not a public address".to_string());
    }

    let response = state
        .http_client
        .get(&url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch: {}", e))?;
    let charset = crate::connectors::response_charset(&response);
    let bytes = crate::connectors::read_body(response)
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    let html = crate::connectors::decode_html(&bytes, charset.as_deref());

    tauri::async_runtime::spawn_blocking(move || extract_content(&html, &url))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No content extracted".to_string())
}

#[cfg(test)]
mod tests {
    use super::{extract_content, extract_with_css_selectors, extract_with_readability};

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

    #[test]
    fn single_paragraph_article_is_accepted() {
        let html = "<html><body><article><p>One paragraph is enough. The reader should show this \
            text instead of failing, because short posts still carry real content worth reading.</p>\
            </article></body></html>";
        assert!(extract_content(html, "https://example.com/note").is_some());
    }

    #[test]
    fn uppercase_markup_is_accepted() {
        let html = "<HTML><BODY><ARTICLE><P>Uppercase tags are valid HTML and must not be treated \
            as a missing paragraph structure, otherwise the reading pane reports no content.</P>\
            </ARTICLE></BODY></HTML>";
        assert!(extract_with_css_selectors(html).is_some());
    }
}
