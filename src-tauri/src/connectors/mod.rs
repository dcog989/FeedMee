pub mod bluesky;
pub mod rss;
pub mod website;

use log::debug;
use scraper::{Html, Selector};
use std::io::Cursor;
use url::Url;

use crate::AppState;

/// Result of feed source detection for addition.
pub enum DetectedFeed {
    Bluesky {
        url: String,
    },
    Rss {
        feed: feed_rs::model::Feed,
        url: String,
    },
    Website {
        url: String,
        content: Vec<u8>,
    },
}

impl DetectedFeed {
    pub async fn add(self, folder_id: Option<i64>, state: &AppState) -> Result<i64, String> {
        match self {
            DetectedFeed::Bluesky { url } => {
                bluesky::add_bluesky_feed(&url, folder_id, state).await
            },
            DetectedFeed::Rss { feed, url } => {
                rss::add_rss_feed(feed, &url, folder_id, state).await
            },
            DetectedFeed::Website { url, content } => {
                website::add_website_feed(&url, &content, folder_id, state).await
            },
        }
    }
}

/// Refresh a feed by its stored type.
pub async fn refresh_feed_by_type(
    feed_type: &str,
    feed_url: &str,
    feed_id: i64,
    state: &AppState,
) -> Result<i64, String> {
    match feed_type {
        "website" => website::refresh_website_feed(feed_url, feed_id, state).await,
        "bluesky" => bluesky::refresh_bluesky_feed(feed_url, feed_id, state).await,
        _ => rss::refresh_rss_feed(feed_url, feed_id, state).await,
    }
}

/// Detect the feed source type from a URL.
/// For Bluesky URLs, returns immediately without HTTP requests.
/// For other URLs, fetches content and tries RSS/Atom parsing,
/// falls back to website scraping if no feed is found.
pub async fn detect_feed(url: &str, client: &reqwest::Client) -> Result<DetectedFeed, String> {
    if bluesky::extract_handle(url).is_some() {
        return Ok(DetectedFeed::Bluesky {
            url: url.to_string(),
        });
    }

    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    let original_url = response.url().clone();
    let content_bytes = response.bytes().await.map_err(|e| e.to_string())?;

    let initial_parse = match feed_rs::parser::parse(Cursor::new(content_bytes.clone())) {
        Ok(f) if !f.entries.is_empty() => Some(f),
        _ => None,
    };

    if let Some(feed) = initial_parse {
        debug!("detect_feed: RSS parse ok for {}", url);
        return Ok(DetectedFeed::Rss {
            feed,
            url: url.to_string(),
        });
    }

    let html = String::from_utf8_lossy(&content_bytes);
    let discovered_url = discover_rss_feed_url(&html, &original_url);

    if let Some(rss_url) = discovered_url {
        debug!("detect_feed: discovered RSS url={}", rss_url);
        let resp = client
            .get(&rss_url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
        match feed_rs::parser::parse(Cursor::new(bytes)) {
            Ok(f) if !f.entries.is_empty() => {
                return Ok(DetectedFeed::Rss {
                    feed: f,
                    url: rss_url,
                });
            },
            _ => {
                debug!("detect_feed: discovered RSS empty or unparseable, treating as website");
            },
        }
    } else {
        debug!("detect_feed: no RSS found, treating as website");
    }

    Ok(DetectedFeed::Website {
        url: url.to_string(),
        content: content_bytes.to_vec(),
    })
}

/// Parse HTML for <link> tags with RSS/Atom feed types and return the best candidate URL.
fn discover_rss_feed_url(html: &str, base_url: &Url) -> Option<String> {
    let feed_types = [
        "application/rss+xml",
        "application/atom+xml",
        "application/feed+json",
    ];
    let document = Html::parse_document(html);
    Selector::parse("link")
        .ok()
        .map(|sel| {
            document
                .select(&sel)
                .filter_map(|el| {
                    let t = el.value().attr("type").unwrap_or("");
                    if feed_types.iter().any(|ft| t.contains(ft)) {
                        el.value()
                            .attr("href")
                            .and_then(|href| base_url.join(href).ok().map(|u| u.to_string()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .and_then(|urls| {
            if urls.is_empty() {
                None
            } else {
                urls.into_iter().max_by_key(|u| u.len())
            }
        })
}
