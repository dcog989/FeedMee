use super::{SAVED_FEED_ID, kept_articles};
use crate::models::Article;
use rusqlite::{Connection, Result, params};
use std::collections::HashSet;

/// Shared projection for article queries. `map_articles` reads columns by
/// positional index, so every query must select exactly this list in order.
const ARTICLE_COLUMNS: &str = "a.id, a.feed_id, a.title, a.author, a.summary, a.url, a.image_url, \
     a.timestamp, a.is_read, a.is_saved, \
     EXISTS (SELECT 1 FROM article_tags WHERE article_id = a.id) AS has_tags";

/// Effective time for views that ask "what is new". Feeds without dates store
/// `timestamp = 0`, which would otherwise hide them from Latest and Today; the
/// row's insert time is a usable stand-in.
const ARTICLE_TIME: &str = "CASE WHEN a.timestamp > 0 THEN a.timestamp ELSE a.created_at END";

fn order_clause(desc: bool) -> &'static str {
    if desc { "DESC" } else { "ASC" }
}

pub fn get_articles_for_feed(
    conn: &Connection,
    feed_id: i64,
    limit: usize,
    offset: usize,
    sort_desc: bool,
) -> Result<Vec<Article>> {
    let order = order_clause(sort_desc);
    let sql = format!(
        "SELECT {ARTICLE_COLUMNS}
         FROM articles a WHERE a.feed_id = ?1
         ORDER BY a.timestamp {order}, a.id {order} LIMIT ?2 OFFSET ?3"
    );
    let mut stmt = conn.prepare(&sql)?;
    map_articles(&mut stmt, params![feed_id, limit as i64, offset as i64])
}

pub fn get_articles_for_folder(
    conn: &Connection,
    folder_id: i64,
    limit: usize,
    offset: usize,
    sort_desc: bool,
) -> Result<Vec<Article>> {
    let order = order_clause(sort_desc);
    let sql = format!(
        "SELECT {ARTICLE_COLUMNS}
         FROM articles a
         JOIN feeds f ON a.feed_id = f.id
         WHERE f.folder_id = ?1 AND f.id != {SAVED_FEED_ID}
         ORDER BY a.timestamp {order}, a.id {order} LIMIT ?2 OFFSET ?3"
    );
    let mut stmt = conn.prepare(&sql)?;
    map_articles(&mut stmt, params![folder_id, limit as i64, offset as i64])
}

pub fn get_latest_articles(
    conn: &Connection,
    cutoff_timestamp: i64,
    limit: usize,
    offset: usize,
    sort_desc: bool,
) -> Result<Vec<Article>> {
    let order = order_clause(sort_desc);
    let sql = format!(
        "SELECT {ARTICLE_COLUMNS}
         FROM articles a WHERE {ARTICLE_TIME} > ?1 AND a.feed_id != {SAVED_FEED_ID}
         ORDER BY {ARTICLE_TIME} {order}, a.id {order} LIMIT ?2 OFFSET ?3"
    );
    let mut stmt = conn.prepare(&sql)?;
    map_articles(&mut stmt, params![cutoff_timestamp, limit as i64, offset as i64])
}

pub fn get_saved_articles(conn: &Connection, limit: usize, offset: usize, sort_desc: bool) -> Result<Vec<Article>> {
    let order = order_clause(sort_desc);
    let sql = format!(
        "SELECT {ARTICLE_COLUMNS}
         FROM articles a WHERE a.is_saved = 1
         ORDER BY a.timestamp {order}, a.id {order} LIMIT ?1 OFFSET ?2"
    );
    let mut stmt = conn.prepare(&sql)?;
    map_articles(&mut stmt, params![limit as i64, offset as i64])
}

fn map_articles(stmt: &mut rusqlite::Statement, params: impl rusqlite::Params) -> Result<Vec<Article>> {
    stmt.query_map(params, |row| {
        Ok(Article {
            id: row.get(0)?,
            feed_id: row.get(1)?,
            title: row.get(2)?,
            author: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            summary: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
            url: row.get(5)?,
            image_url: row.get(6)?,
            image_low_res: false,
            timestamp: row.get::<_, Option<i64>>(7)?.unwrap_or(0),
            is_read: row.get(8)?,
            is_saved: row.get(9)?,
            has_tags: row.get::<_, i64>(10)? != 0,
        })
    })?
    .collect::<Result<Vec<Article>>>()
}

pub fn batch_insert_articles(conn: &mut Connection, articles: &[Article], cutoff: Option<i64>) -> Result<usize> {
    let tx = conn.transaction()?;
    let mut inserter = ArticleInserter::new(&tx, cutoff)?;
    let mut count = 0;
    for article in articles {
        count += inserter.insert(article)?;
    }
    drop(inserter);
    tx.commit()?;
    Ok(count)
}

/// Prepared statements for bulk article upserts. Reuses one statement per
/// operation instead of re-preparing on every row; drop before committing.
struct ArticleInserter<'a> {
    insert: rusqlite::Statement<'a>,
    update_image: rusqlite::Statement<'a>,
    update_summary: rusqlite::Statement<'a>,
    cutoff: Option<i64>,
}

impl<'a> ArticleInserter<'a> {
    fn new(conn: &'a Connection, cutoff: Option<i64>) -> Result<Self> {
        Ok(Self {
            cutoff,
            insert: conn.prepare(
                "INSERT INTO articles (feed_id, title, author, summary, url, image_url, timestamp, created_at, is_read, is_saved)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CAST(strftime('%s','now') AS INTEGER), 0, 0)
                 ON CONFLICT(feed_id, url) DO NOTHING",
            )?,
            update_image: conn.prepare(
                "UPDATE articles SET image_url = ?1
                 WHERE feed_id = ?2 AND url = ?3 AND (image_url = '' OR ?4)
                   AND image_url IS NOT ?1",
            )?,
            update_summary: conn.prepare(
                "UPDATE articles SET summary = ?1
                 WHERE feed_id = ?2 AND url = ?3
                   AND summary IS NOT ?1
                   AND (summary IS NULL OR summary = '' OR length(?1) >= length(summary))",
            )?,
        })
    }

    fn insert(&mut self, article: &Article) -> Result<usize> {
        // Never resurrect articles that are past the retention window; refresh
        // would otherwise re-add rows the startup purge just removed.
        if let Some(cutoff) = self.cutoff
            && article.timestamp != 0
            && article.timestamp < cutoff
        {
            return Ok(0);
        }
        let inserted = self.insert.execute(params![
            article.feed_id,
            article.title,
            article.author,
            article.summary,
            article.url,
            article.image_url,
            article.timestamp,
        ])?;
        if inserted == 0 {
            if !article.image_url.is_empty() {
                self.update_image.execute(params![
                    article.image_url,
                    article.feed_id,
                    article.url,
                    !article.image_low_res,
                ])?;
            }
            if !article.summary.is_empty() {
                // Only fill in or lengthen the summary; a shorter duplicate
                // must not replace the stored copy.
                self.update_summary
                    .execute(params![article.summary, article.feed_id, article.url])?;
            }
        }
        Ok(inserted)
    }
}

/// Subset of `urls` that is already stored for the feed. Only the fetched batch
/// is looked up, so a refresh never loads every URL the feed has stored.
pub fn get_known_urls(conn: &Connection, feed_id: i64, urls: &[String]) -> Result<HashSet<String>> {
    // Chunked because SQLite caps the number of bound parameters.
    const CHUNK: usize = 500;

    let mut known = HashSet::new();
    for chunk in urls.chunks(CHUNK) {
        let placeholders = std::iter::repeat_n("?", chunk.len()).collect::<Vec<_>>().join(",");
        let sql = format!("SELECT url FROM articles WHERE feed_id = ?1 AND url IN ({placeholders})");
        let mut stmt = conn.prepare(&sql)?;

        let mut values: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(chunk.len() + 1);
        values.push(&feed_id);
        values.extend(chunk.iter().map(|url| url as &dyn rusqlite::ToSql));

        let rows = stmt.query_map(values.as_slice(), |row| row.get::<_, String>(0))?;
        known.extend(rows.collect::<Result<Vec<String>>>()?);
    }
    Ok(known)
}

pub fn set_article_read(conn: &Connection, article_id: i64, is_read: bool) -> Result<()> {
    conn.execute(
        "UPDATE articles SET is_read = ?1 WHERE id = ?2",
        params![is_read, article_id],
    )?;
    Ok(())
}

fn mark_read_where(conn: &Connection, filter: &str, param: Option<i64>) -> Result<()> {
    let sql = format!(
        "UPDATE articles SET is_read = 1
         WHERE NOT ({})
           {filter}",
        kept_articles("articles")
    );
    match param {
        Some(id) => conn.execute(&sql, params![id])?,
        None => conn.execute(&sql, [])?,
    };
    Ok(())
}

pub fn mark_feed_read(conn: &Connection, feed_id: i64) -> Result<()> {
    mark_read_where(conn, "AND feed_id = ?1", Some(feed_id))
}

pub fn mark_folder_read(conn: &Connection, folder_id: i64) -> Result<()> {
    // The sentinel feed sits in folder 0 but must never be touched by a
    // folder-wide action.
    let filter = format!(
        "AND feed_id != {SAVED_FEED_ID}
         AND feed_id IN (SELECT id FROM feeds WHERE folder_id = ?1)"
    );
    mark_read_where(conn, &filter, Some(folder_id))
}

pub fn mark_global_read(conn: &Connection) -> Result<()> {
    mark_read_where(conn, "", None)
}

pub fn update_article_saved(conn: &Connection, article_id: i64, is_saved: bool) -> Result<()> {
    if !is_saved {
        // Saved articles from deleted feeds live in the sentinel feed. Unsaving
        // one would leave a row no view can reach, so drop it instead — unless a
        // tag still makes it reachable through search.
        let deleted = conn.execute(
            &format!(
                "DELETE FROM articles
                 WHERE id = ?1 AND feed_id = ?2
                   AND NOT ({})",
                kept_articles("articles")
            ),
            params![article_id, SAVED_FEED_ID],
        )?;
        if deleted > 0 {
            return Ok(());
        }
    }
    conn.execute(
        "UPDATE articles SET is_saved = ?1 WHERE id = ?2",
        params![is_saved as i64, article_id],
    )?;
    Ok(())
}

pub fn search_articles(
    conn: &Connection,
    query: &str,
    limit: usize,
    offset: usize,
    sort_desc: bool,
) -> Result<Vec<Article>> {
    let order = order_clause(sort_desc);
    // Escape each term for FTS5: wrap it in quotes to treat it as a literal
    // phrase, and double any embedded double quotes to prevent operator
    // injection. Terms combine with AND, so word order does not matter.
    let escaped = query
        .split_whitespace()
        .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" AND ");
    if escaped.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "SELECT {ARTICLE_COLUMNS}
         FROM articles_fts
         JOIN articles a ON articles_fts.rowid = a.id
         WHERE articles_fts MATCH ?1
         ORDER BY a.timestamp {order}, a.id {order} LIMIT ?2 OFFSET ?3"
    );
    let mut stmt = conn.prepare(&sql)?;
    map_articles(&mut stmt, params![escaped, limit as i64, offset as i64])
}
