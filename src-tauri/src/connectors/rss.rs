use std::io::Cursor;

use async_trait::async_trait;
use log::{debug, error, info};
use scraper::{Html, Selector};
use url::Url;

use crate::commands::scraper::compute_content_hash;
use crate::{AppState, db, models::Article};

use super::FeedConnector;

pub struct RssConnector;

#[async_trait]
impl FeedConnector for RssConnector {
    fn feed_type(&self) -> &'static str {
        "rss"
    }

    async fn fetch_articles(
        &self,
        url: &str,
        state: &AppState,
    ) -> Result<(String, String, Vec<Article>), String> {
        let client = &state.http_client;
        let response = client.get(url).send().await.map_err(|e| e.to_string())?;
        let original_url = response.url().clone();
        let content_bytes = response.bytes().await.map_err(|e| e.to_string())?;

        if let Ok(feed) = feed_rs::parser::parse(Cursor::new(content_bytes.clone())) {
            if !feed.entries.is_empty() {
                let title = feed
                    .title
                    .as_ref()
                    .map(|t| t.content.clone())
                    .unwrap_or_else(|| "Untitled Feed".to_string());
                let articles = entries_to_articles(feed.entries, 0, url);
                return Ok((title, url.to_string(), articles));
            }
        }

        let html = String::from_utf8_lossy(&content_bytes);
        if let Some(rss_url) = discover_rss_feed_url(&html, &original_url) {
            debug!("rss connector: discovered RSS url={}", rss_url);
            let resp = client
                .get(&rss_url)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
            if let Ok(feed) = feed_rs::parser::parse(Cursor::new(bytes)) {
                if !feed.entries.is_empty() {
                    let title = feed
                        .title
                        .as_ref()
                        .map(|t| t.content.clone())
                        .unwrap_or_else(|| "Untitled Feed".to_string());
                    let articles = entries_to_articles(feed.entries, 0, &rss_url);
                    return Ok((title, rss_url, articles));
                }
            }
        }

        Err("No RSS feed found".to_string())
    }

    async fn refresh(&self, feed_url: &str, feed_id: i64, state: &AppState) -> Result<i64, String> {
        refresh_rss_feed(feed_url, feed_id, state).await
    }
}

pub fn entries_to_articles(
    entries: Vec<feed_rs::model::Entry>,
    feed_id: i64,
    feed_url: &str,
) -> Vec<Article> {
    entries
        .into_iter()
        .map(|entry| {
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
                        feed_url.trim_end_matches('/'),
                        compute_content_hash(&key)
                    )
                });

            let image_url = (|| -> Option<String> {
                if let Some(obj) = entry.media.iter().find_map(|m| m.content.first())
                    && let Some(url) = &obj.url
                {
                    return Some(url.as_str().to_string());
                }
                for link in &entry.links {
                    if link.rel.as_deref() == Some("enclosure")
                        && link
                            .media_type
                            .as_deref()
                            .is_some_and(|m| m.starts_with("image/"))
                    {
                        return Some(link.href.clone());
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
                                return Some(src);
                            }
                            if let Ok(base) = Url::parse(&article_url)
                                && let Ok(abs) = base.join(&src)
                            {
                                return Some(abs.to_string());
                            }
                        }
                    }
                }
                None
            })()
            .unwrap_or_default();

            Article {
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
                    .content
                    .and_then(|c| c.body)
                    .or_else(|| entry.summary.map(|s| s.content))
                    .unwrap_or_default(),
                url: article_url,
                image_url,
                timestamp: entry
                    .published
                    .or(entry.updated)
                    .map(|d| d.timestamp())
                    .unwrap_or(0),
                is_read: false,
                is_saved: false,
                has_tags: false,
            }
        })
        .collect()
}

async fn refresh_rss_feed(feed_url: &str, feed_id: i64, state: &AppState) -> Result<i64, String> {
    let client = state.http_client.clone();
    let result = client.get(feed_url).send().await;

    match result {
        Ok(response) => {
            let content = response
                .bytes()
                .await
                .map_err(|e| format!("Failed to read response: {}", e))?;

            match feed_rs::parser::parse(Cursor::new(content)) {
                Ok(feed) => {
                    info!(
                        "refresh_rss_feed: feed_id={}, {} entries",
                        feed_id,
                        feed.entries.len()
                    );

                    let articles = entries_to_articles(feed.entries, feed_id, feed_url);

                    let conn = state.db.lock().unwrap();
                    conn.execute_batch("BEGIN TRANSACTION")
                        .map_err(|e| e.to_string())?;
                    for article in &articles {
                        let inserted = db::insert_article(&conn, article).unwrap_or(0);
                        if inserted == 0 {
                            if !article.image_url.is_empty() {
                                let _ = db::update_article_image(
                                    &conn,
                                    feed_id,
                                    &article.url,
                                    &article.image_url,
                                );
                            }
                            if !article.summary.is_empty() {
                                let _ = db::update_article_summary(
                                    &conn,
                                    feed_id,
                                    &article.url,
                                    &article.summary,
                                );
                            }
                        }
                    }
                    conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
                    let _ = db::update_feed_error(&conn, feed_id, false);
                    let unread = db::get_feed_unread_count(&conn, feed_id).unwrap_or(0);
                    Ok(unread)
                },
                Err(e) => {
                    error!("refresh_rss_feed: parse error for {}: {}", feed_url, e);
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
