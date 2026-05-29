use crate::{AppState, db, models::Article};
use log::{debug, error, info};
use scraper::{Html, Selector};
use std::io::Cursor;
use tauri::State;
use url::Url;

use super::scraper::{compute_content_hash, scrape_articles_from_page};

async fn add_website_feed(
    url: &str,
    content_bytes: &[u8],
    folder_id: Option<i64>,
    state: &State<'_, AppState>,
) -> Result<i64, String> {
    let html = String::from_utf8_lossy(content_bytes);
    let document = Html::parse_document(&html);
    let title_sel = Selector::parse("title").ok();
    let title = title_sel
        .and_then(|sel| document.select(&sel).next())
        .map(|el| el.text().collect::<String>())
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| url.to_string());

    let feed_id = {
        let conn = state.db.lock().unwrap();
        let target =
            folder_id.unwrap_or_else(|| db::create_folder(&conn, "Uncategorized").unwrap_or(1));
        db::create_feed(&conn, &title, url, target, "website").map_err(|e| e.to_string())?
    };

    let mut articles = scrape_articles_from_page(&html, url);
    for a in &mut articles {
        a.feed_id = feed_id;
    }

    if articles.is_empty() {
        if let Ok(conn) = state.db.lock() {
            let _ = db::delete_feed(&conn, feed_id);
        }
        return Err(format!("No articles found on page: {}", url));
    }

    let conn = state.db.lock().unwrap();
    conn.execute_batch("BEGIN TRANSACTION")
        .map_err(|e| e.to_string())?;
    for article in articles {
        let _ = db::insert_article(&conn, &article);
    }
    conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;

    Ok(feed_id)
}

#[tauri::command]
pub async fn add_feed(
    url: String,
    folder_id: Option<i64>,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    if let Some(handle) = crate::connectors::bluesky::extract_handle(&url) {
        let (did, display_name) =
            crate::connectors::bluesky::resolve_author_info(&state.http_client, &handle).await?;

        let feed_url = format!("bsky:{}", did);

        let feed_id = {
            let conn = state.db.lock().unwrap();
            let target =
                folder_id.unwrap_or_else(|| db::create_folder(&conn, "Uncategorized").unwrap_or(1));
            db::create_feed(&conn, &display_name, &feed_url, target, "bluesky")
                .map_err(|e| e.to_string())?
        };

        let articles =
            crate::connectors::bluesky::fetch_posts(&state.http_client, &did, feed_id).await?;

        let conn = state.db.lock().unwrap();
        conn.execute_batch("BEGIN TRANSACTION")
            .map_err(|e| e.to_string())?;
        for article in &articles {
            let _ = db::insert_article(&conn, article);
        }
        conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
        let _ = db::update_feed_error(&conn, feed_id, false);
        info!(
            "add_feed: bluesky feed_id={}, articles={}",
            feed_id,
            articles.len()
        );

        return Ok(feed_id);
    }

    let client = state.http_client.clone();
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;

    let original_url = response.url().clone();
    let content_bytes = response.bytes().await.map_err(|e| e.to_string())?;

    let initial_parse = match feed_rs::parser::parse(Cursor::new(content_bytes.clone())) {
        Ok(f) if !f.entries.is_empty() => Some((f, url.clone())),
        _ => None,
    };

    debug!(
        "add_feed: url={}, initial_parse={}",
        url,
        initial_parse.is_some()
    );

    let (feed, final_url, feed_type) = if let Some((f, u)) = initial_parse {
        (f, u, "rss".to_string())
    } else {
        let discovered_url_str = {
            let html_content = String::from_utf8_lossy(&content_bytes);
            debug!(
                "add_feed: HTML preview (first 1000): {}",
                &html_content[..html_content.len().min(1000)]
            );
            let document = Html::parse_document(&html_content);
            let feed_types = [
                "application/rss+xml",
                "application/atom+xml",
                "application/feed+json",
            ];
            let base_url = Url::parse(original_url.as_str()).ok();
            let discovered_urls: Vec<String> = Selector::parse("link")
                .ok()
                .map(|sel| {
                    document
                        .select(&sel)
                        .filter_map(|el| {
                            let t = el.value().attr("type").unwrap_or("");
                            if feed_types.iter().any(|ft| t.contains(ft)) {
                                el.value().attr("href").and_then(|href| {
                                    base_url
                                        .as_ref()
                                        .and_then(|base| base.join(href).ok())
                                        .map(|u| u.to_string())
                                })
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            debug!(
                "add_feed: discovered {} RSS feed URLs",
                discovered_urls.len()
            );
            for u in &discovered_urls {
                debug!("add_feed:   RSS candidate: {}", u);
            }

            discovered_urls.into_iter().max_by_key(|u| u.len())
        };

        if let Some(new_url) = discovered_url_str {
            debug!("add_feed: discovered RSS url={}", new_url);
            let resp = client
                .get(&new_url)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
            match feed_rs::parser::parse(Cursor::new(bytes.clone())) {
                Ok(f) => {
                    info!(
                        "add_feed: RSS parse ok, {} entries, title={:?}",
                        f.entries.len(),
                        f.title.as_ref().map(|t| &t.content)
                    );
                    if f.entries.is_empty() {
                        info!(
                            "add_feed: RSS feed is empty, falling back to website scraping for {}",
                            url
                        );
                        return add_website_feed(&url, &content_bytes, folder_id, &state).await;
                    }
                    (f, new_url, "rss".to_string())
                },
                Err(e) => {
                    error!("add_feed: RSS parse failed for {}: {}", new_url, e);
                    let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(500)]);
                    error!("add_feed: response preview: {}", preview);
                    return add_website_feed(&url, &content_bytes, folder_id, &state).await;
                },
            }
        } else {
            debug!("add_feed: no RSS found, treating as website");
            return add_website_feed(&url, &content_bytes, folder_id, &state).await;
        }
    };

    let title = feed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| "Untitled Feed".to_string());

    let id = {
        let conn = state.db.lock().unwrap();
        let target =
            folder_id.unwrap_or_else(|| db::create_folder(&conn, "Uncategorized").unwrap_or(1));
        db::create_feed(&conn, &title, &final_url, target, &feed_type).map_err(|e| e.to_string())?
    };

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
                    final_url.trim_end_matches('/'),
                    compute_content_hash(&key)
                )
            });

        let article = Article {
            id: 0,
            feed_id: id,
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
    let _ = db::update_feed_error(&conn, id, false);

    Ok(id)
}
