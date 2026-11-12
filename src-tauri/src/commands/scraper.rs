use crate::AppState;
use crate::models::Article;
use log::debug;
use scraper::{Html, Selector};
use std::sync::Arc;
use url::Url;

pub fn scrape_og_image_from_html(html: &str, page_url: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let meta_sel = Selector::parse("meta").ok()?;
    let raw = document
        .select(&meta_sel)
        .find_map(|el| {
            let prop = el.value().attr("property").unwrap_or("");
            let name = el.value().attr("name").unwrap_or("");
            if prop == "og:image" || name == "twitter:image" || name == "twitter:image:src" {
                el.value().attr("content").map(str::trim).filter(|c| !c.is_empty())
            } else {
                None
            }
        })?
        .to_string();
    if raw.starts_with("http://") || raw.starts_with("https://") {
        Some(raw)
    } else {
        Url::parse(page_url)
            .ok()
            .and_then(|base| base.join(&raw).ok())
            .filter(|u| matches!(u.scheme(), "http" | "https"))
            .map(|u| u.to_string())
    }
}

pub async fn scrape_og_image(client: &reqwest::Client, article_url: &str) -> Option<String> {
    if !crate::net::is_public_url(article_url) {
        return None;
    }

    let response = client
        .get(article_url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .ok()?;
    let charset = crate::connectors::response_charset(&response);
    let bytes = crate::connectors::read_body(response).await.ok()?;
    let html = crate::connectors::decode_html(&bytes, charset.as_deref());
    let page_url = article_url.to_string();

    tauri::async_runtime::spawn_blocking(move || scrape_og_image_from_html(&html, &page_url))
        .await
        .ok()
        .flatten()
}

/// Stable 64-bit FNV-1a key, used for placeholder ids and cache file names.
pub(crate) fn short_hash(content: &str) -> String {
    fn fnv1a_64(bytes: &[u8]) -> u64 {
        let prime: u64 = 0x100000001b3;
        let mut hash: u64 = 0xcbf29ce484222325;
        for &b in bytes {
            hash ^= b as u64;
            hash = hash.wrapping_mul(prime);
        }
        hash
    }
    format!("{:x}", fnv1a_64(content.as_bytes()))
}

/// Returns true when an anchor lives inside site boilerplate (footer, navigation,
/// or an author byline) rather than article content, so it can be skipped.
fn anchor_is_boilerplate(el: &scraper::ElementRef) -> bool {
    let class = el.attr("class").unwrap_or("").to_lowercase();
    if class
        .split_whitespace()
        .any(|c| c.contains("author") || c.contains("byline"))
    {
        return true;
    }
    let mut node = el.parent();
    while let Some(parent) = node {
        if let Some(element) = parent.value().as_element() {
            let tag = element.name().to_lowercase();
            if tag == "footer" || tag == "nav" || tag == "aside" {
                return true;
            }
            let cls = element.attr("class").unwrap_or("").to_lowercase();
            let id = element.attr("id").unwrap_or("").to_lowercase();
            let role = element.attr("role").unwrap_or("").to_lowercase();
            let hay = format!("{id} {cls} {role}");
            if hay
                .split_whitespace()
                .any(|c| c.contains("author") || c.contains("byline"))
                || hay
                    .split_whitespace()
                    .any(|c| matches!(c, "footer" | "navigation" | "contentinfo" | "nav" | "navbar"))
            {
                return true;
            }
        }
        node = parent.parent();
    }
    false
}

/// True when the anchor sits inside the page's main content, where article
/// links live. Navigation survives `anchor_is_boilerplate` on sites that wrap
/// it in a plain `<header>`, so its location is the stronger signal.
fn anchor_is_in_main_content(el: &scraper::ElementRef) -> bool {
    let mut node = el.parent();
    while let Some(parent) = node {
        if let Some(element) = parent.value().as_element() {
            let tag = element.name();
            if tag == "article" || tag == "main" || element.attr("role") == Some("main") {
                return true;
            }
        }
        node = parent.parent();
    }
    false
}

pub fn scrape_articles_from_page(html: &str, page_url: &str) -> Vec<Article> {
    debug!("scrape_articles_from_page: url={}", page_url);
    let base = match Url::parse(page_url) {
        Ok(u) => u,
        Err(_) => return vec![],
    };
    let base_host = base.host_str().unwrap_or("").to_string();

    let document = Html::parse_document(html);
    let anchor_sel = match Selector::parse("a[href]") {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let mut seen = std::collections::HashSet::new();
    let mut candidates: Vec<(bool, Article)> = Vec::new();

    for el in document.select(&anchor_sel) {
        if anchor_is_boilerplate(&el) {
            continue;
        }

        let href = match el.value().attr("href") {
            Some(h) => h,
            None => continue,
        };

        let abs = match base.join(href) {
            Ok(u) => u,
            Err(_) => continue,
        };

        if abs.host_str().unwrap_or("") != base_host {
            continue;
        }
        if abs.path() == base.path() {
            continue;
        }

        let url_str = abs.to_string();
        if !seen.insert(url_str.clone()) {
            continue;
        }

        let anchor_text: String = el.text().collect::<Vec<_>>().join(" ");
        let anchor_text = anchor_text.split_whitespace().collect::<Vec<_>>().join(" ");

        let title = if anchor_text.len() >= 10 {
            anchor_text.clone()
        } else if let Some(t) = el.value().attr("title").filter(|t| t.len() >= 10) {
            t.to_string()
        } else {
            let slug = abs
                .path_segments()
                .and_then(|mut segs| segs.rfind(|s| !s.is_empty() && s.len() > 3))
                .unwrap_or("");
            let from_slug = slug.replace(['-', '_'], " ");
            if from_slug.len() >= 10 {
                from_slug
                    .split_whitespace()
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                debug!("scrape: skipping, no usable title for {}", url_str);
                continue;
            }
        };

        let path_depth = abs
            .path_segments()
            .map(|s| s.filter(|p| !p.is_empty()).count())
            .unwrap_or(0);
        if path_depth < 2 && anchor_text.len() < 10 {
            debug!("scrape: skipping shallow nav url {}", url_str);
            continue;
        }

        debug!("scrape: accepting {:?} -> {}", title, url_str);
        candidates.push((
            anchor_is_in_main_content(&el),
            Article {
                id: 0,
                feed_id: 0,
                title,
                author: String::new(),
                summary: String::new(),
                url: url_str,
                image_url: String::new(),
                image_low_res: false,
                timestamp: now,
                is_read: false,
                is_saved: false,
                has_tags: false,
            },
        ));
    }

    let mut trusted = Vec::new();
    let mut fallback = Vec::new();
    for (in_main, article) in candidates {
        if in_main {
            trusted.push(article);
        } else {
            fallback.push(article);
        }
    }
    // Sites that wrap everything in one container have no main content marker,
    // so keep the old behaviour instead of returning an empty feed.
    if trusted.is_empty() {
        fallback
    } else {
        trusted
    }
}

pub async fn backfill_og_images<F>(state: &AppState, articles: &mut [Article], mut should_fill: F)
where
    F: FnMut(&Article) -> bool,
{
    const CONCURRENCY: usize = 6;

    let targets: Vec<(usize, String)> = articles
        .iter()
        .enumerate()
        .filter(|(_, a)| (a.image_url.is_empty() || a.image_low_res) && should_fill(a))
        .map(|(idx, a)| (idx, a.url.clone()))
        .collect();

    if targets.is_empty() {
        return;
    }

    let client = state.http_client.clone();
    let semaphore = Arc::clone(&state.http_semaphore);
    let queue = std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::from(targets)));
    let results = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));

    let mut workers = Vec::new();
    for _ in 0..CONCURRENCY {
        let queue = std::sync::Arc::clone(&queue);
        let results = std::sync::Arc::clone(&results);
        let client = client.clone();
        let semaphore = Arc::clone(&semaphore);
        workers.push(tauri::async_runtime::spawn(async move {
            loop {
                let next = queue.lock().unwrap().pop_front();
                let Some((idx, article_url)) = next else {
                    break;
                };
                let Ok(_permit) = semaphore.acquire().await else {
                    break;
                };
                if let Some(img) = scrape_og_image(&client, &article_url).await {
                    results.lock().unwrap().insert(idx, img);
                }
            }
        }));
    }

    for worker in workers {
        let _ = worker.await;
    }

    let filled = std::mem::take(&mut *results.lock().unwrap());
    for (idx, img) in filled {
        articles[idx].image_url = img;
        articles[idx].image_low_res = false;
    }
}

#[cfg(test)]
mod tests {
    use super::{scrape_articles_from_page, scrape_og_image_from_html};

    #[test]
    fn resolves_relative_og_image() {
        let html = r#"<html><head><meta property="og:image" content="/img/cover.png"></head></html>"#;
        assert_eq!(
            scrape_og_image_from_html(html, "https://example.com/post/1").as_deref(),
            Some("https://example.com/img/cover.png")
        );
    }

    #[test]
    fn ignores_empty_og_image() {
        let html = r#"<html><head><meta property="og:image" content="  "></head></html>"#;
        assert_eq!(scrape_og_image_from_html(html, "https://example.com/post/1"), None);
    }

    #[test]
    fn keeps_main_content_links_only() {
        let html = r#"<html><body>
            <header><a href="/about">About the company</a></header>
            <main>
                <article><a href="/posts/one">A real article title</a></article>
                <a href="/posts/two">Another real article</a>
            </main>
        </body></html>"#;
        let articles = scrape_articles_from_page(html, "https://example.com/");
        let urls: Vec<&str> = articles.iter().map(|a| a.url.as_str()).collect();
        assert_eq!(
            urls,
            vec!["https://example.com/posts/one", "https://example.com/posts/two"]
        );
    }

    #[test]
    fn falls_back_to_all_links_without_main_content() {
        let html = r#"<html><body><div><a href="/posts/three">A third real title</a></div></body></html>"#;
        let articles = scrape_articles_from_page(html, "https://example.com/");
        assert_eq!(articles.len(), 1);
    }
}
