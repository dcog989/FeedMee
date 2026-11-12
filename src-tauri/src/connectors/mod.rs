pub mod add;
pub mod bluesky;
pub mod rss;
pub mod website;

use std::sync::OnceLock;

use async_trait::async_trait;

use crate::{AppState, models::Article};

/// A single HTTP download of a feed/page, shared between connectors during
/// discovery so the original URL is only fetched once.
pub struct FetchedPage {
    pub bytes: Vec<u8>,
    pub final_url: url::Url,
    /// Charset declared by the response, for HTML consumers that must decode
    /// the bytes themselves.
    pub charset: Option<String>,
}

/// A feed discovered by a connector, ready to be persisted.
pub struct FetchedFeed {
    pub title: String,
    pub url: String,
    pub articles: Vec<Article>,
    /// Position marker for paged sources, stored with the feed on add.
    pub cursor: Option<String>,
}

/// Refuse oversized pages and feeds so a mis-pasted link or broken server
/// cannot exhaust memory.
const MAX_PAGE_BYTES: usize = 10 * 1024 * 1024;

/// Only the newest entries get an og:image lookup; older entries rarely need a
/// hero, and each lookup is a network request.
const BACKFILL_LIMIT: usize = 10;

pub(crate) async fn read_body(mut response: reqwest::Response) -> Result<Vec<u8>, String> {
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if body.len() + chunk.len() > MAX_PAGE_BYTES {
            return Err(format!("Response is larger than {} bytes", MAX_PAGE_BYTES));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

pub(crate) fn response_charset(response: &reqwest::Response) -> Option<String> {
    let content_type = response.headers().get(reqwest::header::CONTENT_TYPE)?.to_str().ok()?;
    content_type.split(';').find_map(|part| {
        let (key, value) = part.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case("charset")
            .then(|| value.trim().trim_matches('"').to_string())
    })
}

/// Decode a downloaded page. Only the response header is consulted.
///
/// shortcut: a `meta charset` inside the body is ignored, which matters for
/// legacy servers that omit the header; upgrade when such pages show mojibake.
pub(crate) fn decode_html(bytes: &[u8], charset: Option<&str>) -> String {
    let encoding = charset
        .and_then(|label| encoding_rs::Encoding::for_label(label.as_bytes()))
        .unwrap_or(encoding_rs::UTF_8);
    encoding.decode(bytes).0.into_owned()
}

async fn fetch_page(client: &reqwest::Client, url: &str) -> Option<FetchedPage> {
    if !crate::net::is_public_url(url) {
        return None;
    }
    let response = client.get(url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let final_url = response.url().clone();
    let charset = response_charset(&response);
    let bytes = read_body(response).await.ok()?;
    Some(FetchedPage {
        bytes,
        final_url,
        charset,
    })
}

#[async_trait]
pub trait FeedConnector: Send + Sync {
    fn feed_type(&self) -> &'static str;

    /// Whether discovery should pre-fetch the raw page and pass it in, letting
    /// HTML-consuming connectors share one download. Connectors that resolve
    /// their own API endpoints should return `false`.
    fn wants_prefetched_page(&self) -> bool {
        true
    }

    async fn fetch_articles(
        &self,
        url: &str,
        page: Option<&FetchedPage>,
        state: &AppState,
    ) -> Result<FetchedFeed, String>;

    async fn refresh(&self, feed_url: &str, feed_id: i64, state: &AppState) -> Result<i64, String>;
}

pub struct Registry {
    connectors: Vec<Box<dyn FeedConnector>>,
}

impl Registry {
    pub async fn detect_and_add(&self, url: &str, folder_id: Option<i64>, state: &AppState) -> Result<i64, String> {
        let is_bluesky = bluesky::extract_handle(url).is_some();
        let mut page: Option<Option<FetchedPage>> = None;
        let mut last_error: Option<String> = None;

        for connector in &self.connectors {
            // Bluesky owns bsky.app profile URLs; the other connectors own the rest.
            if is_bluesky && connector.feed_type() != "bluesky" {
                continue;
            }
            if !is_bluesky && connector.feed_type() == "bluesky" {
                continue;
            }

            let page_ref = if connector.wants_prefetched_page() {
                if page.is_none() {
                    match fetch_page(&state.http_client, url).await {
                        Some(fetched) => page = Some(Some(fetched)),
                        None => return Err(format!("Could not fetch {}", url)),
                    }
                }
                page.as_ref().and_then(|p| p.as_ref())
            } else {
                None
            };

            match connector.fetch_articles(url, page_ref, state).await {
                Ok(fetched) => {
                    return add::add_feed_with_articles(
                        &fetched.title,
                        &fetched.url,
                        connector.feed_type(),
                        fetched.articles,
                        fetched.cursor,
                        folder_id,
                        state,
                    )
                    .await;
                },
                Err(e) => last_error = Some(e),
            }
        }

        Err(last_error.unwrap_or_else(|| "Could not add feed: unsupported URL".to_string()))
    }

    pub async fn refresh(
        &self,
        feed_type: &str,
        feed_url: &str,
        feed_id: i64,
        state: &AppState,
    ) -> Result<i64, String> {
        for connector in &self.connectors {
            if connector.feed_type() == feed_type {
                return connector.refresh(feed_url, feed_id, state).await;
            }
        }
        Err(format!("Unknown feed type: {}", feed_type))
    }
}

static REGISTRY: OnceLock<Registry> = OnceLock::new();

pub fn registry() -> &'static Registry {
    REGISTRY.get_or_init(|| Registry {
        connectors: vec![
            Box::new(bluesky::BlueskyConnector),
            Box::new(rss::RssConnector),
            Box::new(website::WebsiteConnector),
        ],
    })
}
