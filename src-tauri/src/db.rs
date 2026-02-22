use crate::models::{Article, Feed, Folder};
use log::{debug, info};
use rusqlite::{Connection, Result, params};
use rusqlite_migration::{M, Migrations};

// Each entry is an immutable, append-only migration.
// Never edit a past migration - add a new one instead.
fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        // v1: initial schema
        M::up(
            "CREATE TABLE IF NOT EXISTS folders (
                id   INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            );
            CREATE TABLE IF NOT EXISTS feeds (
                id           INTEGER PRIMARY KEY,
                name         TEXT NOT NULL,
                url          TEXT NOT NULL UNIQUE,
                folder_id    INTEGER NOT NULL,
                has_error    BOOLEAN NOT NULL DEFAULT 0,
                feed_type    TEXT NOT NULL DEFAULT 'rss',
                content_hash TEXT,
                FOREIGN KEY (folder_id) REFERENCES folders (id)
            );
            CREATE TABLE IF NOT EXISTS articles (
                id        INTEGER PRIMARY KEY,
                feed_id   INTEGER NOT NULL,
                title     TEXT NOT NULL,
                author    TEXT,
                summary   TEXT,
                url       TEXT NOT NULL UNIQUE,
                timestamp INTEGER,
                is_read   BOOLEAN NOT NULL DEFAULT 0,
                is_saved  BOOLEAN NOT NULL DEFAULT 0,
                FOREIGN KEY (feed_id) REFERENCES feeds (id)
            );
            INSERT OR IGNORE INTO folders (id, name) VALUES (1, 'Uncategorized');",
        ),
    ])
}

pub fn init_db(conn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing database");

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;",
    )?;

    let m = migrations();
    m.to_latest(conn)?;

    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    info!("Database schema at version {}", version);

    Ok(())
}

pub fn run_vacuum(conn: &Connection) -> Result<()> {
    info!("Running database VACUUM...");
    conn.execute("VACUUM", [])?;
    info!("Database VACUUM completed");
    Ok(())
}

// --- Read Operations ---

pub fn get_folders_with_feeds(conn: &Connection) -> Result<Vec<Folder>> {
    debug!("Querying folders with feeds");

    let mut folder_stmt =
        conn.prepare("SELECT id, name FROM folders ORDER BY name COLLATE NOCASE")?;

    let mut feed_stmt = conn.prepare(
        "SELECT f.id, f.name, f.url, f.folder_id, f.has_error, f.feed_type, f.content_hash,
                (SELECT COUNT(*) FROM articles a WHERE a.feed_id = f.id AND a.is_read = 0) AS unread_count
         FROM feeds f
         WHERE f.folder_id = ?1
         ORDER BY f.name COLLATE NOCASE",
    )?;

    let folders = folder_stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let feeds = feed_stmt
                .query_map([id], |r| {
                    Ok(Feed {
                        id: r.get(0)?,
                        name: r.get(1)?,
                        url: r.get(2)?,
                        folder_id: r.get(3)?,
                        has_error: r.get::<_, bool>(4).unwrap_or(false),
                        feed_type: r.get(5).unwrap_or_else(|_| "rss".to_string()),
                        content_hash: r.get(6).unwrap_or_default(),
                        unread_count: r.get(7)?,
                    })
                })
                .and_then(|rows| rows.collect());
            Ok(Folder {
                id,
                name,
                feeds: feeds.unwrap_or_default(),
            })
        })?
        .collect::<Result<Vec<Folder>>>()?;

    Ok(folders)
}

pub fn get_articles_for_feed(
    conn: &Connection,
    feed_id: i64,
    limit: usize,
    offset: usize,
    sort_asc: bool,
) -> Result<Vec<Article>> {
    let order = if sort_asc { "ASC" } else { "DESC" };
    let sql = format!(
        "SELECT id, feed_id, title, author, summary, url, timestamp, is_read, is_saved
         FROM articles WHERE feed_id = ?1
         ORDER BY timestamp {} LIMIT ?2 OFFSET ?3",
        order
    );
    let mut stmt = conn.prepare(&sql)?;
    map_articles(&mut stmt, params![feed_id, limit as i64, offset as i64])
}

pub fn get_articles_for_folder(
    conn: &Connection,
    folder_id: i64,
    limit: usize,
    offset: usize,
    sort_asc: bool,
) -> Result<Vec<Article>> {
    let order = if sort_asc { "ASC" } else { "DESC" };
    let sql = format!(
        "SELECT a.id, a.feed_id, a.title, a.author, a.summary, a.url, a.timestamp, a.is_read, a.is_saved
         FROM articles a
         JOIN feeds f ON a.feed_id = f.id
         WHERE f.folder_id = ?1
         ORDER BY a.timestamp {} LIMIT ?2 OFFSET ?3",
        order
    );
    let mut stmt = conn.prepare(&sql)?;
    map_articles(&mut stmt, params![folder_id, limit as i64, offset as i64])
}

pub fn get_latest_articles(
    conn: &Connection,
    cutoff_timestamp: i64,
    limit: usize,
    offset: usize,
    sort_asc: bool,
) -> Result<Vec<Article>> {
    let order = if sort_asc { "ASC" } else { "DESC" };
    let sql = format!(
        "SELECT id, feed_id, title, author, summary, url, timestamp, is_read, is_saved
         FROM articles WHERE timestamp > ?1
         ORDER BY timestamp {} LIMIT ?2 OFFSET ?3",
        order
    );
    let mut stmt = conn.prepare(&sql)?;
    map_articles(
        &mut stmt,
        params![cutoff_timestamp, limit as i64, offset as i64],
    )
}

pub fn get_saved_articles(
    conn: &Connection,
    limit: usize,
    offset: usize,
    sort_asc: bool,
) -> Result<Vec<Article>> {
    let order = if sort_asc { "ASC" } else { "DESC" };
    let sql = format!(
        "SELECT id, feed_id, title, author, summary, url, timestamp, is_read, is_saved
         FROM articles WHERE is_saved = 1
         ORDER BY timestamp {} LIMIT ?1 OFFSET ?2",
        order
    );
    let mut stmt = conn.prepare(&sql)?;
    map_articles(&mut stmt, params![limit as i64, offset as i64])
}

fn map_articles(
    stmt: &mut rusqlite::Statement,
    params: impl rusqlite::Params,
) -> Result<Vec<Article>> {
    stmt.query_map(params, |row| {
        Ok(Article {
            id: row.get(0)?,
            feed_id: row.get(1)?,
            title: row.get(2)?,
            author: row.get(3).unwrap_or_default(),
            summary: row.get(4).unwrap_or_default(),
            url: row.get(5)?,
            timestamp: row.get(6)?,
            is_read: row.get(7)?,
            is_saved: row.get(8)?,
        })
    })?
    .collect::<Result<Vec<Article>>>()
}

pub fn get_feed_url(conn: &Connection, feed_id: i64) -> Result<String> {
    conn.query_row(
        "SELECT url FROM feeds WHERE id = ?1",
        params![feed_id],
        |r| r.get(0),
    )
}

pub fn get_feed(conn: &Connection, feed_id: i64) -> Result<Feed> {
    conn.query_row(
        "SELECT id, name, url, folder_id, has_error, feed_type, content_hash FROM feeds WHERE id = ?1",
        params![feed_id],
        |r| Ok(Feed {
            id: r.get(0)?,
            name: r.get(1)?,
            url: r.get(2)?,
            folder_id: r.get(3)?,
            has_error: r.get::<_, bool>(4).unwrap_or(false),
            feed_type: r.get(5).unwrap_or_else(|_| "rss".to_string()),
            content_hash: r.get(6).unwrap_or_default(),
            unread_count: 0,
        }),
    )
}

// --- Write Operations ---

pub fn create_folder(conn: &Connection, name: &str) -> Result<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO folders (name) VALUES (?1)",
        params![name],
    )?;
    conn.query_row(
        "SELECT id FROM folders WHERE name = ?1",
        params![name],
        |r| r.get(0),
    )
}

pub fn create_feed(
    conn: &Connection,
    name: &str,
    url: &str,
    folder_id: i64,
    feed_type: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO feeds (name, url, folder_id, has_error, feed_type) VALUES (?1, ?2, ?3, 0, ?4)
         ON CONFLICT(url) DO UPDATE SET feed_type = excluded.feed_type",
        params![name, url, folder_id, feed_type],
    )?;
    Ok(())
}

pub fn update_feed_error(conn: &Connection, feed_id: i64, has_error: bool) -> Result<()> {
    conn.execute(
        "UPDATE feeds SET has_error = ?1 WHERE id = ?2",
        params![has_error, feed_id],
    )?;
    Ok(())
}

pub fn update_feed_content_hash(conn: &Connection, feed_id: i64, content_hash: &str) -> Result<()> {
    conn.execute(
        "UPDATE feeds SET content_hash = ?1 WHERE id = ?2",
        params![content_hash, feed_id],
    )?;
    Ok(())
}

pub fn insert_article(conn: &Connection, article: &Article) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO articles (feed_id, title, author, summary, url, timestamp, is_read, is_saved)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0)",
        params![article.feed_id, article.title, article.author, article.summary, article.url, article.timestamp],
    )?;
    Ok(())
}

pub fn set_article_read(conn: &Connection, article_id: i64, is_read: bool) -> Result<()> {
    conn.execute(
        "UPDATE articles SET is_read = ?1 WHERE id = ?2",
        params![is_read, article_id],
    )?;
    Ok(())
}

pub fn mark_article_read(conn: &Connection, article_id: i64) -> Result<()> {
    set_article_read(conn, article_id, true)
}

pub fn mark_feed_read(conn: &Connection, feed_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE articles SET is_read = 1 WHERE feed_id = ?1 AND is_saved = 0",
        params![feed_id],
    )?;
    Ok(())
}

pub fn mark_folder_read(conn: &Connection, folder_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE articles SET is_read = 1
         WHERE feed_id IN (SELECT id FROM feeds WHERE folder_id = ?1) AND is_saved = 0",
        params![folder_id],
    )?;
    Ok(())
}

pub fn mark_global_read(conn: &Connection) -> Result<()> {
    conn.execute(
        "UPDATE articles SET is_read = 1 WHERE is_saved = 0",
        [],
    )?;
    Ok(())
}

pub fn update_article_saved(conn: &Connection, article_id: i64, is_saved: bool) -> Result<()> {
    conn.execute(
        "UPDATE articles SET is_saved = ?1 WHERE id = ?2",
        params![is_saved as i64, article_id],
    )?;
    Ok(())
}

// --- Management Operations ---

pub fn rename_folder(conn: &Connection, id: i64, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE folders SET name = ?1 WHERE id = ?2",
        params![new_name, id],
    )?;
    Ok(())
}

pub fn rename_feed(conn: &Connection, id: i64, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE feeds SET name = ?1 WHERE id = ?2",
        params![new_name, id],
    )?;
    Ok(())
}

pub fn delete_feed(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM articles WHERE feed_id = ?1", params![id])?;
    conn.execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn delete_folder(conn: &Connection, id: i64) -> Result<()> {
    let mut stmt = conn.prepare("SELECT id FROM feeds WHERE folder_id = ?1")?;
    let feed_ids: Vec<i64> = stmt
        .query_map(params![id], |row| row.get(0))?
        .collect::<Result<Vec<i64>>>()?;
    for feed_id in feed_ids {
        delete_feed(conn, feed_id)?;
    }
    conn.execute("DELETE FROM folders WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn move_feed(conn: &Connection, feed_id: i64, target_folder_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE feeds SET folder_id = ?1 WHERE id = ?2",
        params![target_folder_id, feed_id],
    )?;
    Ok(())
}

pub fn search_articles(
    conn: &Connection,
    query: &str,
    limit: usize,
    offset: usize,
    sort_asc: bool,
) -> Result<Vec<Article>> {
    let order = if sort_asc { "ASC" } else { "DESC" };
    let pattern = format!("%{}%", query);
    let sql = format!(
        "SELECT id, feed_id, title, author, summary, url, timestamp, is_read, is_saved
         FROM articles WHERE title LIKE ?1
         ORDER BY timestamp {} LIMIT ?2 OFFSET ?3",
        order
    );
    let mut stmt = conn.prepare(&sql)?;
    map_articles(&mut stmt, params![pattern, limit as i64, offset as i64])
}
