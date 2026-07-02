use crate::commands::scraper::{backfill_og_images, compute_content_hash, scrape_og_image};
use crate::{AppState, db, models::Article};
use log::{debug, error, info};
use scraper::{Html, Selector};
use std::io::Cursor;
use url::Url;

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

pub async fn add_rss_feed(
    feed: feed_rs::model::Feed,
    url: &str,
    folder_id: Option<i64>,
    state: &AppState,
) -> Result<i64, String> {
    let title = feed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| "Untitled Feed".to_string());

    let feed_id = {
        let conn = state.db.lock().unwrap();
        if db::feed_exists_by_url(&conn, url).map_err(|e| e.to_string())? {
            return Err("Feed already exists".to_string());
        }
        db::create_feed(&conn, &title, url, folder_id, "rss").map_err(|e| e.to_string())?
    };

    let mut articles = entries_to_articles(feed.entries, feed_id, url);
    backfill_og_images(&state.http_client, &mut articles).await;

    let conn = state.db.lock().unwrap();
    let inserted = db::batch_insert_articles(&conn, &articles).map_err(|e| e.to_string())?;
    let _ = db::update_feed_error(&conn, feed_id, false);

    debug!(
        "add_rss_feed: feed_id={}, new_articles={}",
        feed_id, inserted
    );
    Ok(feed_id)
}

pub async fn refresh_rss_feed(
    feed_url: &str,
    feed_id: i64,
    state: &AppState,
) -> Result<i64, String> {
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

                    let mut articles = entries_to_articles(feed.entries, feed_id, feed_url);

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
