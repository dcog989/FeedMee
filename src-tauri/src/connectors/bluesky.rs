use crate::models::Article;
use log::{debug, info};
use serde::Deserialize;

const PUBLIC_API: &str = "https://public.api.bsky.app";

#[derive(Deserialize)]
struct ResolveHandleResponse {
    did: String,
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
    #[allow(dead_code)]
    indexed_at: Option<String>,
}

#[derive(Deserialize)]
struct Actor {
    handle: String,
    #[allow(dead_code)]
    did: String,
    display_name: Option<String>,
}

/// Extract the handle from a bsky.app/profile/{handle} URL.
/// Returns the handle part, or None if the URL doesn't match.
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

/// Resolve a Bluesky handle to a DID using the public API.
pub async fn resolve_handle(client: &reqwest::Client, handle: &str) -> Result<String, String> {
    let url = format!(
        "{}/xrpc/com.atproto.identity.resolveHandle?handle={}",
        PUBLIC_API, handle
    );
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Bluesky resolve handle network error: {}", e))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "Bluesky resolve handle failed ({}): {}",
            status, body
        ));
    }
    let data: ResolveHandleResponse = resp
        .json()
        .await
        .map_err(|e| format!("Bluesky resolve handle parse error: {}", e))?;
    Ok(data.did)
}

/// Get the display name from an author's first available post.
/// Returns (did, display_name).
pub async fn resolve_author_info(
    client: &reqwest::Client,
    handle: &str,
) -> Result<(String, String), String> {
    let did = resolve_handle(client, handle).await?;
    debug!("resolve_author_info: resolved {} -> {}", handle, did);

    let feed = fetch_author_feed(client, &did, None).await?;
    let display_name = feed
        .feed
        .first()
        .and_then(|v| v.post.author.display_name.clone())
        .unwrap_or_else(|| handle.to_string());

    Ok((did, display_name))
}

/// Fetch an author's feed from the Bluesky public API.
/// Returns a list of Article structs ready for DB insertion.
async fn fetch_author_feed(
    client: &reqwest::Client,
    actor: &str,
    cursor: Option<&str>,
) -> Result<AuthorFeedResponse, String> {
    let mut url = format!(
        "{}/xrpc/app.bsky.feed.getAuthorFeed?actor={}&limit=50",
        PUBLIC_API, actor
    );
    if let Some(c) = cursor {
        url.push_str(&format!("&cursor={}", c));
    }
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Bluesky fetch feed network error: {}", e))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Bluesky fetch feed failed ({}): {}", status, body));
    }
    resp.json()
        .await
        .map_err(|e| format!("Bluesky fetch feed parse error: {}", e))
}

/// Extract the rkey (record key) from an AT Protocol URI.
/// URI format: at://did:plc:xxx/app.bsky.feed.post/rkey
fn extract_rkey(uri: &str) -> String {
    uri.rsplit('/').next().unwrap_or("unknown").to_string()
}

/// Build a bsky.app URL for a given handle and post rkey.
fn build_post_url(handle: &str, rkey: &str) -> String {
    format!("https://bsky.app/profile/{}/post/{}", handle, rkey)
}

/// Format a post text into a display-friendly title (first line or truncated).
fn post_title(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let first_line = trimmed.lines().next().unwrap_or(trimmed);
    let cleaned = first_line.trim();
    if cleaned.len() <= 100 {
        cleaned.to_string()
    } else {
        format!("{}…", &cleaned[..97])
    }
}

/// Detect embed type from the post's record.embed and return a short label.
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

/// Parse a Bluesky datetime string to a unix timestamp.
fn parse_timestamp(ts: &str) -> i64 {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
        dt.timestamp()
    } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S%.fZ") {
        dt.and_utc().timestamp()
    } else {
        0
    }
}

/// Fetch recent posts from a Bluesky author and return them as Articles.
/// No database access — caller is responsible for insertion.
pub async fn fetch_posts(
    client: &reqwest::Client,
    actor: &str,
    feed_id: i64,
) -> Result<Vec<Article>, String> {
    info!("fetch_posts: actor={}, feed_id={}", actor, feed_id);

    let mut articles = Vec::new();
    let mut cursor: Option<String> = None;

    for page in 0..2 {
        let response = fetch_author_feed(client, actor, cursor.as_deref()).await?;

        if response.feed.is_empty() {
            debug!("fetch_posts: no entries on page {}", page);
            break;
        }

        for view in &response.feed {
            let record = &view.post.record;
            let text = record
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let created = record
                .get("createdAt")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
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

            let summary = text.to_string();

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

    info!(
        "fetch_posts: {} articles for actor={}",
        articles.len(),
        actor
    );
    Ok(articles)
}
