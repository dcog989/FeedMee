use crate::{AppState, db, models::Article};
use log::{debug, error, info};
use readabilityrs::{Readability, ReadabilityOptions};
use std::io::Cursor;
use tauri::State;

use super::scraper::{
    backfill_og_images, compute_content_hash, scrape_articles_from_page, scrape_og_image,
    scrape_og_image_from_html,
};

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
        let og_image = scrape_og_image_from_html(&html, &url);
        let mut articles = scrape_articles_from_page(&html, &url);
        for a in &mut articles {
            a.feed_id = feed_id;
            if a.image_url.is_empty() {
                a.image_url = og_image.clone().unwrap_or_default();
            }
        }
        backfill_og_images(&client, &mut articles).await;
        let conn = state.db.lock().unwrap();
        conn.execute_batch("BEGIN TRANSACTION")
            .map_err(|e| e.to_string())?;
        for article in &articles {
            let _ = db::insert_article(&conn, article);
        }
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

                    let mut articles: Vec<Article> = feed
                        .entries
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
                                        url.trim_end_matches('/'),
                                        compute_content_hash(&key)
                                    )
                                });

                            let image_url = (|| -> Option<String> {
                                if let Some(obj) =
                                    entry.media.iter().find_map(|m| m.content.first())
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
                                // Fallback: extract first image from content or summary HTML
                                let html_sources = [
                                    entry.content.as_ref().and_then(|c| c.body.as_deref()),
                                    entry.summary.as_ref().map(|s| s.content.as_str()),
                                ];
                                for html in html_sources.into_iter().flatten() {
                                    use scraper::{Html, Selector};
                                    if let Ok(sel) = Selector::parse("img[src]") {
                                        let doc = Html::parse_fragment(html);
                                        if let Some(el) = doc.select(&sel).next()
                                            && let Some(src) = el.value().attr("src")
                                        {
                                            let src = src.to_string();
                                            if src.starts_with("http://")
                                                || src.starts_with("https://")
                                            {
                                                return Some(src);
                                            }
                                            if let Ok(base) = url::Url::parse(&article_url)
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
                        .collect();

                    // Eagerly fetch og:image for articles that the feed didn't provide one for.
                    // Spawn all fetches concurrently; the HTTP client's connection pool limits
                    // actual parallelism naturally.
                    let handles: Vec<(usize, tauri::async_runtime::JoinHandle<Option<String>>)> =
                        articles
                            .iter()
                            .enumerate()
                            .filter(|(_, a)| a.image_url.is_empty())
                            .map(|(idx, a)| {
                                let client = client.clone();
                                let article_url = a.url.clone();
                                let handle = tauri::async_runtime::spawn(async move {
                                    scrape_og_image(&client, &article_url).await
                                });
                                (idx, handle)
                            })
                            .collect();

                    for (idx, handle) in handles {
                        if let Ok(Some(img)) = handle.await {
                            articles[idx].image_url = img;
                        }
                    }

                    let conn = state.db.lock().unwrap();
                    conn.execute_batch("BEGIN TRANSACTION")
                        .map_err(|e| e.to_string())?;
                    for article in &articles {
                        let inserted = db::insert_article(&conn, article).unwrap_or(0);
                        // Article already existed — patch image and summary with newly available data.
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
