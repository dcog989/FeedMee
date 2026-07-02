use async_trait::async_trait;
use log::{debug, info, warn};
use serde::Deserialize;

use crate::{AppState, db, models::Article};

use super::FeedConnector;

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

pub struct BlueskyConnector;

#[async_trait]
impl FeedConnector for BlueskyConnector {
    fn feed_type(&self) -> &'static str {
        "bluesky"
    }

    async fn fetch_articles(
        &self,
        url: &str,
        state: &AppState,
    ) -> Result<(String, String, Vec<Article>), String> {
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

fn extract_rkey(uri: &str) -> String {
    uri.rsplit('/').next().unwrap_or("unknown").to_string()
}

fn build_post_url(handle: &str, rkey: &str) -> String {
    format!("https://bsky.app/profile/{}/post/{}", handle, rkey)
}

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

    'outer: for page in 0..2 {
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
    Ok((articles, first_uri))
}

pub async fn resolve_bluesky_source(
    url: &str,
    client: &reqwest::Client,
) -> Result<(String, String, Vec<Article>), String> {
    let handle = extract_handle(url).ok_or_else(|| "Not a Bluesky URL".to_string())?;
    let (did, display_name) = resolve_author_info(client, &handle).await?;
    let feed_url = format!("bsky:{}", did);
    let (articles, _) = fetch_posts(client, &did, 0, None).await?;
    Ok((display_name, feed_url, articles))
}

pub async fn refresh_bluesky_feed(
    feed_url: &str,
    feed_id: i64,
    state: &AppState,
) -> Result<i64, String> {
    let actor = feed_url.strip_prefix("bsky:").unwrap_or(feed_url);

    let last_seen = {
        let conn = state.db.lock().unwrap();
        db::get_bluesky_cursor(&conn, feed_id).unwrap_or(None)
    };

    let (articles, new_cursor) =
        fetch_posts(&state.http_client, actor, feed_id, last_seen.as_deref()).await?;

    let conn = state.db.lock().unwrap();
    let _ = db::batch_insert_articles(&conn, &articles).map_err(|e| e.to_string())?;
    let _ = db::update_feed_error(&conn, feed_id, false);

    if let Some(ref uri) = new_cursor
        && let Err(e) = db::set_bluesky_cursor(&conn, feed_id, uri)
    {
        warn!("Failed to store bluesky cursor for feed {}: {}", feed_id, e);
    }

    Ok(db::get_feed_unread_count(&conn, feed_id).unwrap_or(0))
}
