use std::borrow::Cow;
use std::io::Cursor;
use std::time::Duration;

use async_trait::async_trait;
use log::{debug, error};
use scraper::{Html, Selector};
use url::Url;

use crate::commands::scraper::{backfill_og_images, compute_content_hash};
use crate::{AppState, db, models::Article};

use super::{FeedConnector, FetchedFeed, FetchedPage};

pub struct RssConnector;

#[async_trait]
impl FeedConnector for RssConnector {
    fn feed_type(&self) -> &'static str {
        "rss"
    }

    async fn fetch_articles(
        &self,
        url: &str,
        page: Option<&FetchedPage>,
        state: &AppState,
    ) -> Result<FetchedFeed, String> {
        let client = &state.http_client;
        let (content_bytes, original_url): (Cow<'_, [u8]>, Url) = match page {
            Some(p) => (Cow::Borrowed(p.bytes.as_slice()), p.final_url.clone()),
            None => {
                let response = client.get(url).send().await.map_err(|e| e.to_string())?;
                let final_url = response.url().clone();
                let bytes = response.bytes().await.map_err(|e| e.to_string())?;
                (Cow::Owned(bytes.to_vec()), final_url)
            },
        };

        if let Some(feed) = parse_feed(content_bytes.as_ref(), url) {
            return Ok(feed);
        }

        let html = String::from_utf8_lossy(content_bytes.as_ref());
        let mut candidates = Vec::new();
        if let Some(rss_url) = discover_rss_feed_url(&html, &original_url) {
            debug!("rss connector: discovered RSS url={}", rss_url);
            candidates.push(rss_url);
        }
        candidates.extend(candidate_feed_urls(&original_url));

        for candidate in candidates {
            if let Some(feed) = fetch_feed(client, &candidate).await {
                debug!("rss connector: probed RSS url={}", candidate);
                return Ok(feed);
            }
        }

        Err("No RSS feed found".to_string())
    }

    async fn refresh(&self, feed_url: &str, feed_id: i64, state: &AppState) -> Result<i64, String> {
        refresh_rss_feed(feed_url, feed_id, state).await
    }
}

/// Timeout for individual feed-probe requests, kept short so that probing many
/// conventional paths for an unsupported site stays responsive.
const FEED_PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// Conventional feed paths, appended to a page's directory when HTML
/// autodiscovery yields nothing (e.g. Cloudflare-protected homepages).
const FEED_PATH_CANDIDATES: &[&str] = &[
    "feed",
    "feed/",
    "rss",
    "rss/",
    "rss.xml",
    "feed.xml",
    "atom.xml",
    "index.xml",
];

fn parse_feed(content: &[u8], feed_url: &str) -> Option<FetchedFeed> {
    let feed = feed_rs::parser::parse(Cursor::new(content)).ok()?;
    if feed.entries.is_empty() {
        return None;
    }
    let title = feed
        .title
        .as_ref()
        .map(|t| t.content.clone())
        .unwrap_or_else(|| "Untitled Feed".to_string());
    Some(FetchedFeed {
        title,
        url: feed_url.to_string(),
        articles: entries_to_articles(&feed.entries, 0, feed_url),
    })
}

async fn fetch_feed(client: &reqwest::Client, url: &str) -> Option<FetchedFeed> {
    let response = client.get(url).timeout(FEED_PROBE_TIMEOUT).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let bytes = response.bytes().await.ok()?;
    parse_feed(&bytes, url)
}

/// Build the conventional feed URLs to probe for a page, relative to the
/// page's own directory.
fn candidate_feed_urls(base_url: &Url) -> Vec<String> {
    let mut base = base_url.clone();
    base.set_query(None);
    base.set_fragment(None);
    if !base.path().ends_with('/') {
        let parent = base.path().rsplit_once('/').map(|(head, _)| head).unwrap_or("");
        base.set_path(&format!("{}/", parent));
    }
    FEED_PATH_CANDIDATES
        .iter()
        .filter_map(|suffix| base.join(suffix).ok().map(|u| u.to_string()))
        .collect()
}

pub fn entries_to_articles(entries: &[feed_rs::model::Entry], feed_id: i64, feed_url: &str) -> Vec<Article> {
    entries
        .iter()
        .map(|entry| {
            let article_url = resolve_article_url(entry, feed_url);

            let (image_url, image_low_res) = resolve_image(entry, &article_url);

            Article {
                id: 0,
                feed_id,
                title: entry
                    .title
                    .as_ref()
                    .map(|t| t.content.clone())
                    .unwrap_or_else(|| "No Title".to_string()),
                author: entry.authors.first().map(|p| p.name.clone()).unwrap_or_default(),
                summary: entry
                    .content
                    .as_ref()
                    .and_then(|c| c.body.clone())
                    .or_else(|| entry.summary.as_ref().map(|s| s.content.clone()))
                    .unwrap_or_default(),
                url: article_url,
                image_url,
                image_low_res,
                timestamp: entry.published.or(entry.updated).map(|d| d.timestamp()).unwrap_or(0),
                is_read: false,
                is_saved: false,
                has_tags: false,
            }
        })
        .collect()
}

/// Images smaller than this are considered too low-resolution to render as a
/// full-width hero and are flagged for og:image backfill.
const MIN_ACCEPTABLE_IMAGE_WIDTH: u32 = 400;

fn resolve_image(entry: &feed_rs::model::Entry, article_url: &str) -> (String, bool) {
    let media = entry
        .media
        .iter()
        .flat_map(|m| m.content.iter())
        .filter_map(|c| c.url.as_ref().map(|url| (url.as_str(), c.width)))
        .max_by_key(|(url, declared)| image_width_hint(url, *declared));

    if let Some((url, declared)) = media {
        let low_res = image_width_hint(url, declared).is_some_and(|w| w < MIN_ACCEPTABLE_IMAGE_WIDTH);
        return (url.to_string(), low_res);
    }

    for link in &entry.links {
        if link.rel.as_deref() == Some("enclosure")
            && link.media_type.as_deref().is_some_and(|m| m.starts_with("image/"))
        {
            return (link.href.clone(), false);
        }
    }

    let html_sources = [
        entry.content.as_ref().and_then(|c| c.body.as_deref()),
        entry.summary.as_ref().map(|s| s.content.as_str()),
    ];
    for html in html_sources.into_iter().flatten() {
        if let Ok(sel) = Selector::parse("img[src]") {
            let doc = Html::parse_fragment(html);
            if let Some(el) = doc.select(&sel).next()
                && let Some(src) = el.value().attr("src")
            {
                let src = src.to_string();
                if src.starts_with("http://") || src.starts_with("https://") {
                    return (src, false);
                }
                if let Ok(base) = Url::parse(article_url)
                    && let Ok(abs) = base.join(&src)
                {
                    return (abs.to_string(), false);
                }
            }
        }
    }

    (String::new(), false)
}

/// Best-effort width for an image: the feed's declared `media:content` width,
/// falling back to a `width`/`w` query parameter used by common image CDNs.
fn image_width_hint(url: &str, declared: Option<u32>) -> Option<u32> {
    if declared.is_some() {
        return declared;
    }
    Url::parse(url).ok().and_then(|parsed| {
        parsed
            .query_pairs()
            .find(|(key, _)| key == "width" || key == "w")
            .and_then(|(_, value)| value.parse::<u32>().ok())
    })
}

fn compute_placeholder_url(feed_url: &str, entry: &feed_rs::model::Entry) -> String {
    let key = if !entry.id.is_empty() {
        entry.id.clone()
    } else {
        entry.title.as_ref().map(|t| t.content.clone()).unwrap_or_default()
    };
    format!("{}/#{}", feed_url.trim_end_matches('/'), compute_content_hash(&key))
}

fn resolve_article_url(entry: &feed_rs::model::Entry, feed_url: &str) -> String {
    if let Some(link) = entry
        .links
        .iter()
        .find(|l| l.rel.as_deref() == Some("alternate"))
        .or(entry.links.first())
    {
        return strip_tracking_params(&link.href);
    }
    if let Ok(parsed) = Url::parse(&entry.id)
        && matches!(parsed.scheme(), "http" | "https")
    {
        return strip_tracking_params(&entry.id);
    }
    compute_placeholder_url(feed_url, entry)
}

async fn refresh_rss_feed(feed_url: &str, feed_id: i64, state: &AppState) -> Result<i64, String> {
    let client = state.http_client.clone();
    let result = client.get(feed_url).send().await;

    match result {
        Ok(response) => {
            let status = response.status();
            if !status.is_success() {
                error!(
                    "refresh_rss_feed: HTTP {} for {} (blocked or unavailable)",
                    status, feed_url
                );
                return Err(format!("HTTP {} for {}", status, feed_url));
            }

            let content = response
                .bytes()
                .await
                .map_err(|e| format!("Failed to read response: {}", e))?;

            match feed_rs::parser::parse(Cursor::new(content)) {
                Ok(feed) => {
                    debug!("refresh_rss_feed: feed_id={}, {} entries", feed_id, feed.entries.len());

                    let mut articles = entries_to_articles(&feed.entries, feed_id, feed_url);

                    let known_urls: std::collections::HashSet<String> = {
                        let conn = state.db.lock().unwrap();
                        db::get_article_urls(&conn, feed_id)
                            .map_err(|e| e.to_string())?
                            .into_iter()
                            .collect()
                    };

                    backfill_og_images(state, &mut articles, |a| {
                        a.image_low_res || !known_urls.contains(&a.url)
                    })
                    .await;

                    let mut conn = state.db.lock().unwrap();
                    let tx = conn.transaction().map_err(|e| e.to_string())?;
                    let mut inserter = db::ArticleInserter::new(&tx).map_err(|e| e.to_string())?;
                    for article in &articles {
                        inserter.insert(article).map_err(|e| e.to_string())?;
                    }
                    drop(inserter);
                    tx.commit().map_err(|e| e.to_string())?;
                    let unread = db::get_feed_unread_count(&conn, feed_id).map_err(|e| e.to_string())?;
                    Ok(unread)
                },
                Err(e) => {
                    error!("refresh_rss_feed: parse error for {}: {}", feed_url, e);
                    Err(format!("Parse error: {}", e))
                },
            }
        },
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

fn discover_rss_feed_url(html: &str, base_url: &Url) -> Option<String> {
    let feed_types = ["application/rss+xml", "application/atom+xml", "application/feed+json"];
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

const TRACKING_PARAMS: &[&str] = &[
    "access_token",
    "token",
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "mc_cid",
    "mc_eid",
];

fn strip_tracking_params(url: &str) -> String {
    let mut parsed = match Url::parse(url) {
        Ok(u) => u,
        Err(_) => return url.to_string(),
    };
    let kept: Vec<(String, String)> = parsed
        .query_pairs()
        .filter(|(k, _)| !TRACKING_PARAMS.contains(&k.as_ref()))
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    parsed.set_query(None);
    for (key, value) in kept {
        parsed.query_pairs_mut().append_pair(&key, &value);
    }
    parsed.to_string()
}

#[cfg(test)]
mod tests {
    use super::{MIN_ACCEPTABLE_IMAGE_WIDTH, candidate_feed_urls, image_width_hint};

    #[test]
    fn candidates_are_relative_to_page_directory() {
        let base = url::Url::parse("https://www.neowin.net/news/").unwrap();
        let candidates = candidate_feed_urls(&base);
        assert!(candidates.contains(&"https://www.neowin.net/news/rss/".to_string()));
        assert!(candidates.contains(&"https://www.neowin.net/news/feed".to_string()));
        assert!(candidates.iter().all(|u| u.starts_with("https://www.neowin.net/news/")));
    }

    #[test]
    fn candidates_drop_filename_and_query() {
        let base = url::Url::parse("https://example.com/blog/post?utm_source=x").unwrap();
        let candidates = candidate_feed_urls(&base);
        assert!(candidates.contains(&"https://example.com/blog/feed".to_string()));
        assert!(candidates.contains(&"https://example.com/blog/rss.xml".to_string()));
    }

    #[test]
    fn candidates_for_root_page_use_root_directory() {
        let base = url::Url::parse("https://example.com").unwrap();
        let candidates = candidate_feed_urls(&base);
        assert!(candidates.contains(&"https://example.com/feed".to_string()));
        assert!(candidates.contains(&"https://example.com/rss".to_string()));
    }

    #[test]
    fn width_hint_prefers_declared_value() {
        assert_eq!(
            image_width_hint("https://example.com/img.jpg?width=9999", Some(140)),
            Some(140)
        );
    }

    #[test]
    fn width_hint_falls_back_to_query_param() {
        assert_eq!(
            image_width_hint("https://example.com/img.jpg?width=140", None),
            Some(140)
        );
        assert_eq!(image_width_hint("https://example.com/img.jpg?w=800", None), Some(800));
        assert_eq!(image_width_hint("https://example.com/img.jpg", None), None);
    }

    #[test]
    fn low_res_threshold_matches_guardian_thumbnail() {
        let guardian = "https://i.guim.co.uk/img/media/abc/master/6336.jpg?width=140&quality=85&auto=format&fit=max";
        assert_eq!(image_width_hint(guardian, Some(140)), Some(140));
        assert!(image_width_hint(guardian, Some(140)).unwrap() < MIN_ACCEPTABLE_IMAGE_WIDTH);
    }
}
