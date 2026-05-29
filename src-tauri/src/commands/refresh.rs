use crate::{AppState, db, models::Article};
use log::{debug, error, info};
use readabilityrs::{Readability, ReadabilityOptions};
use std::io::Cursor;
use tauri::State;

use super::scraper::{compute_content_hash, scrape_articles_from_page};

#[tauri::command]
pub async fn get_article_content(
    url: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let client = state.http_client.clone();
    let html = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let options = ReadabilityOptions::default();
    let readability =
        Readability::new(&html, Some(&url), Some(options)).map_err(|e| format!("{:?}", e))?;
    let article = readability.parse().ok_or("Failed to parse content")?;
    article.content.ok_or("No content extracted".to_string())
}

#[tauri::command]
pub async fn refresh_feed(feed_id: i64, state: State<'_, AppState>) -> Result<i64, String> {
    let (url, feed_type) = {
        let conn = state.db.lock().unwrap();
        let feed = db::get_feed(&conn, feed_id).map_err(|e| e.to_string())?;
        (feed.url, feed.feed_type)
    };

    let client = state.http_client.clone();

    let is_website = feed_type == "website" || feed_type.is_empty();
    debug!(
        "refresh_feed: feed_id={}, url={}, feed_type='{}', is_website={}",
        feed_id, url, feed_type, is_website
    );

    if is_website {
        let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
        let html = response.text().await.map_err(|e| e.to_string())?;
        let articles = scrape_articles_from_page(&html, &url);
        let conn = state.db.lock().unwrap();
        conn.execute_batch("BEGIN TRANSACTION")
            .map_err(|e| e.to_string())?;
        let _: usize = articles
            .into_iter()
            .filter_map(|a| db::insert_article(&conn, &a).ok())
            .sum();
        conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
        let _ = db::update_feed_error(&conn, feed_id, false);
        return Ok(db::get_feed_unread_count(&conn, feed_id).unwrap_or(0));
    }

    if feed_type == "bluesky" {
        let actor = url.strip_prefix("bsky:").unwrap_or(&url);
        let articles = crate::connectors::bluesky::fetch_posts(&client, actor, feed_id).await?;
        let conn = state.db.lock().unwrap();
        conn.execute_batch("BEGIN TRANSACTION")
            .map_err(|e| e.to_string())?;
        let _: usize = articles
            .iter()
            .filter_map(|a| db::insert_article(&conn, a).ok())
            .sum();
        conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
        let _ = db::update_feed_error(&conn, feed_id, false);
        return Ok(db::get_feed_unread_count(&conn, feed_id).unwrap_or(0));
    }

    let result = client.get(&url).send().await;

    match result {
        Ok(response) => {
            let content = response.bytes().await.map_err(|e| e.to_string())?;
            match feed_rs::parser::parse(Cursor::new(content)) {
                Ok(feed) => {
                    info!(
                        "refresh_feed: parsed feed ok, {} entries",
                        feed.entries.len()
                    );
                    let conn = state.db.lock().unwrap();
                    conn.execute_batch("BEGIN TRANSACTION")
                        .map_err(|e| e.to_string())?;
                    for entry in feed.entries {
                        let article_url = entry
                            .links
                            .iter()
                            .find(|l| l.rel.as_deref() == Some("alternate"))
                            .or(entry.links.first())
                            .map(|l| l.href.clone())
                            .unwrap_or_else(|| {
                                let key = if !entry.id.is_empty() {
                                    entry.id.clone()
                                } else {
                                    entry
                                        .title
                                        .as_ref()
                                        .map(|t| t.content.clone())
                                        .unwrap_or_default()
                                };
                                format!(
                                    "{}/#{}",
                                    url.trim_end_matches('/'),
                                    compute_content_hash(&key)
                                )
                            });

                        let article = Article {
                            id: 0,
                            feed_id,
                            title: entry
                                .title
                                .map(|t| t.content)
                                .unwrap_or_else(|| "No Title".to_string()),
                            author: entry
                                .authors
                                .first()
                                .map(|p| p.name.clone())
                                .unwrap_or_default(),
                            summary: entry
                                .summary
                                .map(|s| s.content)
                                .or(entry.content.map(|c| c.body.unwrap_or_default()))
                                .unwrap_or_default(),
                            url: article_url,
                            timestamp: entry
                                .published
                                .or(entry.updated)
                                .map(|d| d.timestamp())
                                .unwrap_or(0),
                            is_read: false,
                            is_saved: false,
                            has_tags: false,
                        };
                        let _ = db::insert_article(&conn, &article);
                    }
                    conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
                    let _ = db::update_feed_error(&conn, feed_id, false);
                    let unread = db::get_feed_unread_count(&conn, feed_id).unwrap_or(0);
                    Ok(unread)
                },
                Err(e) => {
                    error!("refresh_feed: feed_rs parse error for {}: {}", url, e);
                    let conn = state.db.lock().unwrap();
                    let _ = db::update_feed_error(&conn, feed_id, true);
                    Err(format!("Parse error: {}", e))
                },
            }
        },
        Err(e) => {
            let conn = state.db.lock().unwrap();
            let _ = db::update_feed_error(&conn, feed_id, true);
            Err(format!("Network error: {}", e))
        },
    }
}
