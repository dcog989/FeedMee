use super::SAVED_FEED_ID;
use crate::models::Article;
use rusqlite::{Connection, Result, params};

/// Shared projection for article queries. `map_articles` reads columns by
/// positional index, so every query must select exactly this list in order.
const ARTICLE_COLUMNS: &str = "a.id, a.feed_id, a.title, a.author, a.summary, a.url, a.image_url, \
     a.timestamp, a.is_read, a.is_saved, \
     EXISTS (SELECT 1 FROM article_tags WHERE article_id = a.id) AS has_tags";

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
         FROM articles a WHERE a.timestamp > ?1
         ORDER BY a.timestamp {order}, a.id {order} LIMIT ?2 OFFSET ?3"
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
            author: row.get(3).unwrap_or_default(),
            summary: row.get(4).unwrap_or_default(),
            url: row.get(5)?,
            image_url: row.get(6).unwrap_or_default(),
            image_low_res: false,
            timestamp: row.get(7)?,
            is_read: row.get(8)?,
            is_saved: row.get(9)?,
            has_tags: row.get::<_, i64>(10).unwrap_or(0) != 0,
        })
    })?
    .collect::<Result<Vec<Article>>>()
}

pub fn batch_insert_articles(conn: &mut Connection, articles: &[Article]) -> Result<usize> {
    let tx = conn.transaction()?;
    let mut inserter = ArticleInserter::new(&tx)?;
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
pub struct ArticleInserter<'a> {
    insert: rusqlite::Statement<'a>,
    update_image: rusqlite::Statement<'a>,
    update_summary: rusqlite::Statement<'a>,
}

impl<'a> ArticleInserter<'a> {
    pub fn new(conn: &'a Connection) -> Result<Self> {
        Ok(Self {
            insert: conn.prepare(
                "INSERT OR IGNORE INTO articles (feed_id, title, author, summary, url, image_url, timestamp, created_at, is_read, is_saved)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CAST(strftime('%s','now') AS INTEGER), 0, 0)",
            )?,
            update_image: conn.prepare(
                "UPDATE articles SET image_url = ?1
                 WHERE feed_id = ?2 AND url = ?3 AND (image_url = '' OR ?4)",
            )?,
            update_summary: conn.prepare(
                "UPDATE articles SET summary = ?1 WHERE feed_id = ?2 AND url = ?3 AND summary IS NOT ?1",
            )?,
        })
    }

    pub fn insert(&mut self, article: &Article) -> Result<usize> {
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
                self.update_summary
                    .execute(params![article.summary, article.feed_id, article.url])?;
            }
        }
        Ok(inserted)
    }
}

pub fn get_article_urls(conn: &Connection, feed_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT url FROM articles WHERE feed_id = ?1")?;
    let rows = stmt.query_map(params![feed_id], |row| row.get::<_, String>(0))?;
    rows.collect()
}

pub fn set_article_read(conn: &Connection, article_id: i64, is_read: bool) -> Result<()> {
    conn.execute(
        "UPDATE articles SET is_read = ?1 WHERE id = ?2",
        params![is_read, article_id],
    )?;
    Ok(())
}

pub fn mark_feed_read(conn: &Connection, feed_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE articles SET is_read = 1 WHERE feed_id = ?1 AND is_saved = 0 AND NOT EXISTS (SELECT 1 FROM article_tags WHERE article_id = articles.id)",
        params![feed_id],
    )?;
    Ok(())
}

pub fn mark_folder_read(conn: &Connection, folder_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE articles SET is_read = 1
         WHERE feed_id IN (SELECT id FROM feeds WHERE folder_id = ?1) AND is_saved = 0 AND NOT EXISTS (SELECT 1 FROM article_tags WHERE article_id = articles.id)",
        params![folder_id],
    )?;
    Ok(())
}

pub fn mark_global_read(conn: &Connection) -> Result<()> {
    conn.execute("UPDATE articles SET is_read = 1 WHERE is_saved = 0 AND NOT EXISTS (SELECT 1 FROM article_tags WHERE article_id = articles.id)", [])?;
    Ok(())
}

pub fn update_article_saved(conn: &Connection, article_id: i64, is_saved: bool) -> Result<()> {
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
    // Escape the query for FTS5: wrap in quotes to treat as a literal phrase,
    // and double any embedded double quotes to prevent operator injection.
    let escaped = format!("\"{}\"", query.replace('"', "\"\""));
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
