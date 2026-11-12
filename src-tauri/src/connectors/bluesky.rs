use async_trait::async_trait;
use log::{debug, info, warn};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::{AppState, db, models::Article};

use super::{FeedConnector, FetchedFeed, FetchedPage};

const PUBLIC_API: &str = "https://public.api.bsky.app";
const PAGE_SIZE: &str = "50";
/// Pages fetched when a feed is first added, before a cursor exists.
const INITIAL_PAGES: usize = 2;
/// Page cap for a refresh that has to catch up to its stored cursor.
const MAX_CATCHUP_PAGES: usize = 20;

#[derive(Deserialize)]
struct ResolveHandleResponse {
    did: String,
}

#[derive(Deserialize)]
struct ProfileResponse {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct AuthorFeedResponse {
    feed: Vec<FeedViewPost>,
    cursor: Option<String>,
}

#[derive(Deserialize)]
struct FeedViewPost {
    post: PostView,
}

#[derive(Deserialize)]
struct PostView {
    uri: String,
    author: Actor,
    record: serde_json::Value,
}

#[derive(Deserialize)]
struct Actor {
    handle: String,
    display_name: Option<String>,
}

pub struct BlueskyConnector;

#[async_trait]
impl FeedConnector for BlueskyConnector {
    fn feed_type(&self) -> &'static str {
        "bluesky"
    }

    fn wants_prefetched_page(&self) -> bool {
        false
    }

    async fn fetch_articles(
        &self,
        url: &str,
        _page: Option<&FetchedPage>,
        state: &AppState,
    ) -> Result<FetchedFeed, String> {
        resolve_bluesky_source(url, &state.http_client).await
    }

    async fn refresh(&self, feed_url: &str, feed_id: i64, state: &AppState) -> Result<i64, String> {
        refresh_bluesky_feed(feed_url, feed_id, state).await
    }
}

pub fn extract_handle(url: &str) -> Option<String> {
    let url = url.trim_end_matches('/');
    let prefix = "https://bsky.app/profile/";
    if let Some(handle) = url.strip_prefix(prefix)
        && !handle.is_empty()
        && !handle.contains('/')
    {
        return Some(handle.to_string());
    }
    None
}

pub async fn resolve_handle(client: &reqwest::Client, handle: &str) -> Result<String, String> {
    let data: ResolveHandleResponse = get_json(
        client,
        "/xrpc/com.atproto.identity.resolveHandle",
        &[("handle", handle)],
    )
    .await?;
    Ok(data.did)
}

pub async fn resolve_author_info(client: &reqwest::Client, handle: &str) -> Result<(String, String), String> {
    let did = if handle.starts_with("did:") {
        handle.to_string()
    } else {
        resolve_handle(client, handle).await?
    };
    debug!("resolve_author_info: resolved {} -> {}", handle, did);

    let profile: ProfileResponse =
        get_json(client, "/xrpc/app.bsky.actor.getProfile", &[("actor", did.as_str())]).await?;
    let display_name = profile
        .display_name
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| handle.to_string());

    Ok((did, display_name))
}

/// Send one XRPC query and deserialize the JSON response. `reqwest` encodes the
/// query values, so cursors and handles containing reserved characters survive.
async fn get_json<T: DeserializeOwned>(
    client: &reqwest::Client,
    path: &str,
    query: &[(&str, &str)],
) -> Result<T, String> {
    let resp = client
        .get(format!("{}{}", PUBLIC_API, path))
        .query(query)
        .send()
        .await
        .map_err(|e| format!("Bluesky request to {} failed: {}", path, e))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Bluesky request to {} failed ({}): {}", path, status, body));
    }
    resp.json()
        .await
        .map_err(|e| format!("Bluesky response from {} parse error: {}", path, e))
}

async fn fetch_author_feed(
    client: &reqwest::Client,
    actor: &str,
    cursor: Option<&str>,
) -> Result<AuthorFeedResponse, String> {
    let mut query = vec![("actor", actor), ("limit", PAGE_SIZE)];
    if let Some(c) = cursor {
        query.push(("cursor", c));
    }
    get_json(client, "/xrpc/app.bsky.feed.getAuthorFeed", &query).await
}

fn extract_rkey(uri: &str) -> String {
    uri.rsplit('/').next().unwrap_or("unknown").to_string()
}

fn build_post_url(handle: &str, rkey: &str) -> String {
    format!("https://bsky.app/profile/{}/post/{}", handle, rkey)
}

/// Post text is plain text, but the reading pane renders the summary as HTML.
fn escape_post_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\n', "<br>")
}

fn post_title(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let first_line = trimmed.lines().next().unwrap_or(trimmed);
    let cleaned = first_line.trim();
    if cleaned.chars().count() <= 100 {
        cleaned.to_string()
    } else {
        let truncated: String = cleaned.chars().take(97).collect();
        format!("{}…", truncated)
    }
}

fn embed_label(record: &serde_json::Value) -> Option<&'static str> {
    let embed = record.get("embed")?;
    let type_str = embed.get("$type")?.as_str()?;
    match type_str {
        "app.bsky.embed.images" => Some("📷 Image"),
        "app.bsky.embed.video" => Some("🎬 Video"),
        "app.bsky.embed.external" => Some("🔗 Link"),
        "app.bsky.embed.record" => Some("🔁 Quote"),
        _ => None,
    }
}

fn parse_timestamp(ts: &str) -> i64 {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
        dt.timestamp()
    } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S%.fZ") {
        dt.and_utc().timestamp()
    } else {
        0
    }
}

pub async fn fetch_posts(
    client: &reqwest::Client,
    actor: &str,
    feed_id: i64,
    last_seen_uri: Option<&str>,
) -> Result<(Vec<Article>, Option<String>), String> {
    info!("fetch_posts: actor={}, feed_id={}", actor, feed_id);

    let mut articles = Vec::new();
    let mut cursor: Option<String> = None;
    let mut first_uri: Option<String> = None;

    // First add grabs a short history; a refresh keeps paging until it reaches
    // the stored cursor so a long break does not drop posts.
    let max_pages = if last_seen_uri.is_some() {
        MAX_CATCHUP_PAGES
    } else {
        INITIAL_PAGES
    };

    'outer: for page in 0..max_pages {
        let response = fetch_author_feed(client, actor, cursor.as_deref()).await?;

        if response.feed.is_empty() {
            debug!("fetch_posts: no entries on page {}", page);
            break;
        }

        for view in &response.feed {
            if let Some(ref seen) = last_seen_uri
                && view.post.uri == *seen
            {
                debug!("fetch_posts: caught up at post {}", seen);
                break 'outer;
            }

            if first_uri.is_none() {
                first_uri = Some(view.post.uri.clone());
            }

            let record = &view.post.record;
            let text = record.get("text").and_then(|v| v.as_str()).unwrap_or_default();
            let created = record.get("createdAt").and_then(|v| v.as_str()).unwrap_or_default();
            let handle = &view.post.author.handle;
            let display_name = view.post.author.display_name.as_deref().unwrap_or(handle);

            let rkey = extract_rkey(&view.post.uri);
            let post_url = build_post_url(handle, &rkey);

            let title = {
                let t = post_title(text);
                if !t.is_empty() {
                    t
                } else if let Some(label) = embed_label(record) {
                    label.to_string()
                } else {
                    "(post)".to_string()
                }
            };

            let summary = escape_post_text(text);

            let timestamp = if !created.is_empty() {
                parse_timestamp(created)
            } else {
                0
            };

            articles.push(Article {
                id: 0,
                feed_id,
                title,
                author: display_name.to_string(),
                summary,
                url: post_url,
                image_url: String::new(),
                image_low_res: false,
                timestamp,
                is_read: false,
                is_saved: false,
                has_tags: false,
            });
        }

        cursor = response.cursor;
        if cursor.is_none() {
            break;
        }
    }

    info!("fetch_posts: {} articles for actor={}", articles.len(), actor);
    Ok((articles, first_uri))
}

pub async fn resolve_bluesky_source(url: &str, client: &reqwest::Client) -> Result<FetchedFeed, String> {
    let handle = extract_handle(url).ok_or_else(|| "Not a Bluesky URL".to_string())?;
    let (did, display_name) = resolve_author_info(client, &handle).await?;
    let feed_url = format!("bsky:{}", did);
    let (articles, cursor) = fetch_posts(client, &did, 0, None).await?;
    Ok(FetchedFeed {
        title: display_name,
        url: feed_url,
        articles,
        cursor,
    })
}

pub async fn refresh_bluesky_feed(feed_url: &str, feed_id: i64, state: &AppState) -> Result<i64, String> {
    let actor = feed_url.strip_prefix("bsky:").unwrap_or(feed_url);

    let last_seen = {
        let conn = state.db.lock().unwrap();
        db::get_bluesky_cursor(&conn, feed_id).unwrap_or(None)
    };

    let (articles, new_cursor) = fetch_posts(&state.http_client, actor, feed_id, last_seen.as_deref()).await?;

    let cutoff = state.article_retention_cutoff();
    let mut conn = state.db.lock().unwrap();
    let _ = db::batch_insert_articles(&mut conn, &articles, cutoff).map_err(|e| e.to_string())?;

    if let Some(ref uri) = new_cursor
        && let Err(e) = db::set_bluesky_cursor(&conn, feed_id, uri)
    {
        warn!("Failed to store bluesky cursor for feed {}: {}", feed_id, e);
    }

    Ok(db::get_feed_unread_count(&conn, feed_id).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::{escape_post_text, extract_handle, post_title};

    #[test]
    fn short_titles_are_unchanged() {
        assert_eq!(post_title("Hello world"), "Hello world");
        assert_eq!(post_title("  spaced  "), "spaced");
        assert_eq!(post_title(""), "");
    }

    #[test]
    fn long_ascii_titles_truncate_with_ellipsis() {
        let title = post_title(&"a".repeat(150));
        assert_eq!(title.chars().count(), 98);
        assert!(title.ends_with('…'));
    }

    #[test]
    fn multibyte_titles_do_not_panic() {
        let title = post_title(&"🎉".repeat(150));
        assert_eq!(title.chars().count(), 98);
        assert!(title.ends_with('…'));
    }

    #[test]
    fn post_text_is_escaped_for_html() {
        assert_eq!(escape_post_text("a < b & c\nd"), "a &lt; b &amp; c<br>d");
    }

    #[test]
    fn extract_handle_accepts_profile_urls() {
        assert_eq!(
            extract_handle("https://bsky.app/profile/alice.bsky.social").as_deref(),
            Some("alice.bsky.social")
        );
        assert_eq!(
            extract_handle("https://bsky.app/profile/alice.bsky.social/").as_deref(),
            Some("alice.bsky.social")
        );
        assert_eq!(
            extract_handle("https://bsky.app/profile/did:plc:abc123").as_deref(),
            Some("did:plc:abc123")
        );
    }

    #[test]
    fn extract_handle_rejects_other_urls() {
        assert_eq!(
            extract_handle("https://bsky.app/profile/alice.bsky.social/post/3kabc"),
            None
        );
        assert_eq!(extract_handle("https://example.com/profile/alice"), None);
    }
}
