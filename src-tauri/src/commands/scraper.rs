use crate::models::Article;
use log::debug;
use scraper::{Html, Selector};
use url::Url;

pub fn scrape_og_image_from_html(html: &str, page_url: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let meta_sel = Selector::parse("meta").ok()?;
    let raw = document.select(&meta_sel).find_map(|el| {
        let prop = el.value().attr("property").unwrap_or("");
        let name = el.value().attr("name").unwrap_or("");
        if prop == "og:image" || name == "twitter:image" || name == "twitter:image:src" {
            el.value().attr("content").map(str::to_string)
        } else {
            None
        }
    })?;
    if raw.starts_with("http://") || raw.starts_with("https://") {
        Some(raw)
    } else {
        Url::parse(page_url)
            .ok()
            .and_then(|base| base.join(&raw).ok())
            .map(|u| u.to_string())
    }
}

pub async fn scrape_og_image(client: &reqwest::Client, article_url: &str) -> Option<String> {
    let html = client
        .get(article_url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let document = Html::parse_document(&html);
    let meta_sel = Selector::parse("meta").ok()?;

    let raw = document.select(&meta_sel).find_map(|el| {
        let prop = el.value().attr("property").unwrap_or("");
        let name = el.value().attr("name").unwrap_or("");
        if prop == "og:image" || name == "twitter:image" || name == "twitter:image:src" {
            el.value().attr("content").map(str::to_string)
        } else {
            None
        }
    })?;

    if raw.starts_with("http://") || raw.starts_with("https://") {
        Some(raw)
    } else {
        Url::parse(article_url)
            .ok()
            .and_then(|base| base.join(&raw).ok())
            .map(|u| u.to_string())
    }
}

pub fn compute_content_hash(content: &str) -> String {
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
    let mut articles = Vec::new();

    for el in document.select(&anchor_sel) {
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
        articles.push(Article {
            id: 0,
            feed_id: 0,
            title,
            author: String::new(),
            summary: String::new(),
            url: url_str,
            image_url: String::new(),
            timestamp: now,
            is_read: false,
            is_saved: false,
            has_tags: false,
        });
    }

    articles
}

pub async fn backfill_og_images<F>(
    client: &reqwest::Client,
    articles: &mut [Article],
    mut should_fill: F,
) where
    F: FnMut(&Article) -> bool,
{
    const CONCURRENCY: usize = 6;

    let targets: Vec<(usize, String)> = articles
        .iter()
        .enumerate()
        .filter(|(_, a)| a.image_url.is_empty() && should_fill(a))
        .map(|(idx, a)| (idx, a.url.clone()))
        .collect();

    if targets.is_empty() {
        return;
    }

    let queue = std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::from(
        targets,
    )));
    let results = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));

    let mut workers = Vec::new();
    for _ in 0..CONCURRENCY {
        let queue = std::sync::Arc::clone(&queue);
        let results = std::sync::Arc::clone(&results);
        let client = client.clone();
        workers.push(tauri::async_runtime::spawn(async move {
            loop {
                let next = queue.lock().unwrap().pop_front();
                let Some((idx, article_url)) = next else {
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
    }
}
