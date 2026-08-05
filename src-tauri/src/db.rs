use crate::models::{Article, Feed, Folder, Tag};
use log::{debug, info};
use rusqlite::{Connection, Result, params};
use rusqlite_migration::{M, Migrations};

pub const DB_FILENAME: &str = "feedmee.sqlite";

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(
            "CREATE TABLE IF NOT EXISTS folders (
                id   INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            );
            INSERT OR IGNORE INTO folders (id, name) VALUES (1, 'Uncategorized');
            CREATE TABLE IF NOT EXISTS feeds (
                id           INTEGER PRIMARY KEY,
                name         TEXT NOT NULL,
                url          TEXT NOT NULL,
                folder_id    INTEGER NOT NULL,
                has_error    BOOLEAN NOT NULL DEFAULT 0,
                feed_type    TEXT NOT NULL DEFAULT 'rss',
                FOREIGN KEY (folder_id) REFERENCES folders (id)
            );
            CREATE TABLE IF NOT EXISTS articles (
                id        INTEGER PRIMARY KEY,
                feed_id   INTEGER NOT NULL,
                title     TEXT NOT NULL,
                author    TEXT,
                summary   TEXT,
                url       TEXT NOT NULL,
                timestamp INTEGER,
                is_read   BOOLEAN NOT NULL DEFAULT 0,
                is_saved  BOOLEAN NOT NULL DEFAULT 0,
                FOREIGN KEY (feed_id) REFERENCES feeds (id),
                UNIQUE(feed_id, url)
            );
            CREATE TABLE IF NOT EXISTS tags (
                id    INTEGER PRIMARY KEY,
                name  TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#4899ec'
            );
            CREATE TABLE IF NOT EXISTS article_tags (
                article_id INTEGER NOT NULL,
                tag_id     INTEGER NOT NULL,
                PRIMARY KEY (article_id, tag_id),
                FOREIGN KEY (article_id) REFERENCES articles(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS articles_fts USING fts5(
                title, author, summary,
                content='articles', content_rowid='id'
            );
            INSERT INTO articles_fts(rowid, title, author, summary)
                SELECT id, title, COALESCE(author,''), COALESCE(summary,'')
                FROM articles;
            CREATE TRIGGER IF NOT EXISTS articles_ai AFTER INSERT ON articles BEGIN
                INSERT INTO articles_fts(rowid, title, author, summary)
                    VALUES (new.id, new.title, COALESCE(new.author,''), COALESCE(new.summary,''));
            END;
            CREATE TRIGGER IF NOT EXISTS articles_ad AFTER DELETE ON articles BEGIN
                INSERT INTO articles_fts(articles_fts, rowid, title, author, summary)
                    VALUES ('delete', old.id, old.title, COALESCE(old.author,''), COALESCE(old.summary,''));
            END;
            CREATE TRIGGER IF NOT EXISTS articles_au AFTER UPDATE ON articles BEGIN
                INSERT INTO articles_fts(articles_fts, rowid, title, author, summary)
                    VALUES ('delete', old.id, old.title, COALESCE(old.author,''), COALESCE(old.summary,''));
                INSERT INTO articles_fts(rowid, title, author, summary)
                    VALUES (new.id, new.title, COALESCE(new.author,''), COALESCE(new.summary,''));
            END;
            CREATE INDEX IF NOT EXISTS idx_articles_feed_timestamp ON articles(feed_id, timestamp);",
        ),
        M::up("ALTER TABLE articles ADD COLUMN image_url TEXT NOT NULL DEFAULT '';"),
        M::up(
            "DROP TRIGGER IF EXISTS articles_au;
             CREATE TRIGGER IF NOT EXISTS articles_au AFTER UPDATE ON articles
             WHEN old.title IS NOT new.title OR old.author IS NOT new.author OR old.summary IS NOT new.summary
             BEGIN
                 INSERT INTO articles_fts(articles_fts, rowid, title, author, summary)
                     VALUES ('delete', old.id, old.title, COALESCE(old.author,''), COALESCE(old.summary,''));
                 INSERT INTO articles_fts(rowid, title, author, summary)
                     VALUES (new.id, new.title, COALESCE(new.author,''), COALESCE(new.summary,''));
             END;",
        ),
        M::up(
            "INSERT OR IGNORE INTO folders (id, name) VALUES (0, '');
             UPDATE feeds SET folder_id = 0 WHERE folder_id = 1;
             DELETE FROM folders WHERE id = 1;",
        ),
        M::up("ALTER TABLE feeds ADD COLUMN bluesky_cursor TEXT;"),
        M::up("CREATE INDEX IF NOT EXISTS idx_articles_timestamp ON articles(timestamp);"),
    ])
}

pub fn init_db(conn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing database");

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;
         PRAGMA cache_size = -64000;
         PRAGMA mmap_size = 268435456;
         PRAGMA journal_size_limit = 67108864;",
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

pub fn purge_old_articles(conn: &Connection, retention_days: u64) -> Result<usize> {
    let cutoff = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        - (retention_days as i64 * 86400);
    let count = conn.execute(
        "DELETE FROM articles WHERE timestamp < ?1 AND timestamp > 0 AND is_saved = 0 AND NOT EXISTS (SELECT 1 FROM article_tags WHERE article_id = articles.id)",
        params![cutoff],
    )?;
    if count > 0 {
        info!(
            "Purged {} old articles (retention: {} days)",
            count, retention_days
        );
    }
    Ok(count)
}

fn feed_derived_fields(url: &str, feed_type: &str) -> (String, String) {
    if feed_type == "bluesky" {
        let did = url.strip_prefix("bsky:").unwrap_or(url);
        (String::new(), did.to_string())
    } else {
        (url.to_string(), url.to_string())
    }
}

// --- Read Operations ---

pub fn get_folders_with_feeds(conn: &Connection) -> Result<Vec<Folder>> {
    debug!("Querying folders with feeds");

    let mut folder_stmt =
        conn.prepare("SELECT id, name FROM folders WHERE id != 0 ORDER BY name COLLATE NOCASE")?;

    let mut feed_stmt = conn.prepare(
        "SELECT f.id, f.name, f.url, f.folder_id, f.has_error, f.feed_type,
                COALESCE(uc.unread_count, 0) AS unread_count
         FROM feeds f
         LEFT JOIN (
             SELECT feed_id, COUNT(*) AS unread_count
             FROM articles
             WHERE is_read = 0
             GROUP BY feed_id
         ) uc ON f.id = uc.feed_id
         WHERE f.folder_id = ?1
         ORDER BY f.name COLLATE NOCASE",
    )?;

    let mut root_feed_stmt = conn.prepare(
        "SELECT f.id, f.name, f.url, f.folder_id, f.has_error, f.feed_type,
                COALESCE(uc.unread_count, 0) AS unread_count
         FROM feeds f
         LEFT JOIN (
             SELECT feed_id, COUNT(*) AS unread_count
             FROM articles
             WHERE is_read = 0
             GROUP BY feed_id
         ) uc ON f.id = uc.feed_id
         WHERE f.folder_id = 0
         ORDER BY f.name COLLATE NOCASE",
    )?;

    let root_feeds: Vec<Feed> = root_feed_stmt
        .query_map([], |r| {
            let raw_fid: i64 = r.get(3)?;
            let url_str: String = r.get(2)?;
            let feed_type_str: String = r.get(5).unwrap_or_else(|_| "rss".to_string());
            let (display_url, source_id) = feed_derived_fields(&url_str, &feed_type_str);
            Ok(Feed {
                id: r.get(0)?,
                name: r.get(1)?,
                url: url_str,
                folder_id: if raw_fid == 0 { None } else { Some(raw_fid) },
                has_error: r.get::<_, bool>(4).unwrap_or(false),
                feed_type: feed_type_str,
                unread_count: r.get(6)?,
                display_url,
                source_id,
            })
        })
        .and_then(|rows| rows.collect())?;

    let mut folders: Vec<Folder> = folder_stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let feeds: Vec<Feed> = feed_stmt
                .query_map([id], |r| {
                    let raw_fid: i64 = r.get(3)?;
                    let url_str: String = r.get(2)?;
                    let feed_type_str: String = r.get(5).unwrap_or_else(|_| "rss".to_string());
                    let (display_url, source_id) = feed_derived_fields(&url_str, &feed_type_str);
                    Ok(Feed {
                        id: r.get(0)?,
                        name: r.get(1)?,
                        url: url_str,
                        folder_id: if raw_fid == 0 { None } else { Some(raw_fid) },
                        has_error: r.get::<_, bool>(4).unwrap_or(false),
                        feed_type: feed_type_str,
                        unread_count: r.get(6)?,
                        display_url,
                        source_id,
                    })
                })
                .and_then(|rows| rows.collect())?;
            Ok(Folder { id, name, feeds })
        })?
        .collect::<Result<Vec<Folder>>>()?;

    if !root_feeds.is_empty() {
        folders.push(Folder {
            id: 0,
            name: String::new(),
            feeds: root_feeds,
        });
    }

    Ok(folders)
}

pub fn get_articles_for_feed(
    conn: &Connection,
    feed_id: i64,
    limit: usize,
    offset: usize,
    sort_desc: bool,
) -> Result<Vec<Article>> {
    let order = if sort_desc { "DESC" } else { "ASC" };
    let sql = format!(
        "SELECT id, feed_id, title, author, summary, url, image_url, timestamp, is_read, is_saved,
                EXISTS (SELECT 1 FROM article_tags WHERE article_id = articles.id) AS has_tags
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
    sort_desc: bool,
) -> Result<Vec<Article>> {
    let order = if sort_desc { "DESC" } else { "ASC" };
    let sql = format!(
        "SELECT a.id, a.feed_id, a.title, a.author, a.summary, a.url, a.image_url, a.timestamp, a.is_read, a.is_saved,
                EXISTS (SELECT 1 FROM article_tags WHERE article_id = a.id) AS has_tags
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
    sort_desc: bool,
) -> Result<Vec<Article>> {
    let order = if sort_desc { "DESC" } else { "ASC" };
    let sql = format!(
        "SELECT id, feed_id, title, author, summary, url, image_url, timestamp, is_read, is_saved,
                EXISTS (SELECT 1 FROM article_tags WHERE article_id = articles.id) AS has_tags
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
    sort_desc: bool,
) -> Result<Vec<Article>> {
    let order = if sort_desc { "DESC" } else { "ASC" };
    let sql = format!(
        "SELECT id, feed_id, title, author, summary, url, image_url, timestamp, is_read, is_saved,
                EXISTS (SELECT 1 FROM article_tags WHERE article_id = articles.id) AS has_tags
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
            image_url: row.get(6).unwrap_or_default(),
            timestamp: row.get(7)?,
            is_read: row.get(8)?,
            is_saved: row.get(9)?,
            has_tags: row.get::<_, i64>(10).unwrap_or(0) != 0,
        })
    })?
    .collect::<Result<Vec<Article>>>()
}

pub fn get_feed_unread_count(conn: &Connection, feed_id: i64) -> Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM articles WHERE feed_id = ?1 AND is_read = 0",
        params![feed_id],
        |r| r.get(0),
    )
}
pub fn get_feed(conn: &Connection, feed_id: i64) -> Result<Feed> {
    conn.query_row(
        "SELECT id, name, url, folder_id, has_error, feed_type,
                (SELECT COUNT(*) FROM articles a WHERE a.feed_id = feeds.id AND a.is_read = 0) AS unread_count
         FROM feeds WHERE id = ?1",
        params![feed_id],
        |r| {
            let raw_fid: i64 = r.get(3)?;
            let url_str: String = r.get(2)?;
            let feed_type_str: String = r.get(5).unwrap_or_else(|_| "rss".to_string());
            let (display_url, source_id) = feed_derived_fields(&url_str, &feed_type_str);
            Ok(Feed {
                id: r.get(0)?,
                name: r.get(1)?,
                url: url_str,
                folder_id: if raw_fid == 0 { None } else { Some(raw_fid) },
                has_error: r.get::<_, bool>(4).unwrap_or(false),
                feed_type: feed_type_str,
                unread_count: r.get(6)?,
                display_url,
                source_id,
            })
        },
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

pub fn feed_exists_by_url(conn: &Connection, url: &str) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM feeds WHERE url = ?1)",
        params![url],
        |r| r.get(0),
    )
}

pub fn create_feed(
    conn: &Connection,
    name: &str,
    url: &str,
    folder_id: Option<i64>,
    feed_type: &str,
) -> Result<i64> {
    let fid = folder_id.unwrap_or(0);
    conn.execute(
        "INSERT INTO feeds (name, url, folder_id, has_error, feed_type) VALUES (?1, ?2, ?3, 0, ?4)",
        params![name, url, fid, feed_type],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_feed_error(conn: &Connection, feed_id: i64, has_error: bool) -> Result<()> {
    conn.execute(
        "UPDATE feeds SET has_error = ?1 WHERE id = ?2",
        params![has_error, feed_id],
    )?;
    Ok(())
}

pub fn get_bluesky_cursor(conn: &Connection, feed_id: i64) -> Result<Option<String>> {
    let result: Result<String> = conn.query_row(
        "SELECT bluesky_cursor FROM feeds WHERE id = ?1",
        params![feed_id],
        |row| row.get(0),
    );
    match result {
        Ok(cursor) if !cursor.is_empty() => Ok(Some(cursor)),
        _ => Ok(None),
    }
}

pub fn set_bluesky_cursor(conn: &Connection, feed_id: i64, cursor: &str) -> Result<()> {
    conn.execute(
        "UPDATE feeds SET bluesky_cursor = ?1 WHERE id = ?2",
        params![cursor, feed_id],
    )?;
    Ok(())
}

pub fn batch_insert_articles(conn: &mut Connection, articles: &[Article]) -> Result<usize> {
    let tx = conn.transaction()?;
    let mut count = 0;
    for article in articles {
        count += insert_article(&tx, article)?;
    }
    tx.commit()?;
    Ok(count)
}

pub fn insert_article(conn: &Connection, article: &Article) -> Result<usize> {
    let inserted = conn.execute(
        "INSERT OR IGNORE INTO articles (feed_id, title, author, summary, url, image_url, timestamp, is_read, is_saved)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, 0)",
        params![article.feed_id, article.title, article.author, article.summary, article.url, article.image_url, article.timestamp],
    )?;
    Ok(inserted)
}

pub fn get_article_urls(conn: &Connection, feed_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT url FROM articles WHERE feed_id = ?1")?;
    let rows = stmt.query_map(params![feed_id], |row| row.get::<_, String>(0))?;
    rows.collect()
}

pub fn update_article_image(
    conn: &Connection,
    feed_id: i64,
    url: &str,
    image_url: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE articles SET image_url = ?1 WHERE feed_id = ?2 AND url = ?3 AND image_url = ''",
        params![image_url, feed_id, url],
    )?;
    Ok(())
}

pub fn update_article_summary(
    conn: &Connection,
    feed_id: i64,
    url: &str,
    summary: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE articles SET summary = ?1 WHERE feed_id = ?2 AND url = ?3 AND summary IS NOT ?1",
        params![summary, feed_id, url],
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

// --- Management Operations ---

pub fn rename_folder(conn: &Connection, id: i64, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE folders SET name = ?1 WHERE id = ?2",
        params![new_name, id],
    )?;
    Ok(())
}

pub fn rename_feed(conn: &Connection, id: i64, new_name: &str, new_url: &str) -> Result<()> {
    conn.execute(
        "UPDATE feeds SET name = ?1, url = ?2 WHERE id = ?3",
        params![new_name, new_url, id],
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

pub fn move_feed(conn: &Connection, feed_id: i64, target_folder_id: Option<i64>) -> Result<()> {
    let fid = target_folder_id.unwrap_or(0);
    conn.execute(
        "UPDATE feeds SET folder_id = ?1 WHERE id = ?2",
        params![fid, feed_id],
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
    // Escape the query for FTS5: wrap in quotes to treat as a literal phrase,
    // and double any embedded double quotes to prevent operator injection.
    let escaped = format!("\"{}\"", query.replace('"', "\"\""));
    let sql = format!(
        "SELECT a.id, a.feed_id, a.title, a.author, a.summary, a.url, a.image_url, a.timestamp, a.is_read, a.is_saved,
                EXISTS (SELECT 1 FROM article_tags WHERE article_id = a.id) AS has_tags
         FROM articles_fts
         JOIN articles a ON articles_fts.rowid = a.id
         WHERE articles_fts MATCH ?1
         ORDER BY a.timestamp {} LIMIT ?2 OFFSET ?3",
        order
    );
    let mut stmt = conn.prepare(&sql)?;
    map_articles(&mut stmt, params![escaped, limit as i64, offset as i64])
}

// --- Tag Operations ---

pub fn get_tags_for_article(conn: &Connection, article_id: i64) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color
         FROM tags t
         JOIN article_tags at ON t.id = at.tag_id
         WHERE at.article_id = ?1
         ORDER BY t.name COLLATE NOCASE",
    )?;
    let tags = stmt
        .query_map(params![article_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<Tag>>>()?;
    Ok(tags)
}

pub fn get_all_tags(conn: &Connection) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare("SELECT id, name, color FROM tags ORDER BY name COLLATE NOCASE")?;
    let tags = stmt
        .query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<Tag>>>()?;
    Ok(tags)
}

pub fn add_tag_to_article(
    conn: &Connection,
    article_id: i64,
    name: &str,
    color: &str,
) -> Result<Tag> {
    conn.execute(
        "INSERT OR IGNORE INTO tags (name, color) VALUES (?1, ?2)",
        params![name, color],
    )?;
    let tag_id: i64 =
        conn.query_row("SELECT id FROM tags WHERE name = ?1", params![name], |r| {
            r.get(0)
        })?;
    conn.execute(
        "INSERT OR IGNORE INTO article_tags (article_id, tag_id) VALUES (?1, ?2)",
        params![article_id, tag_id],
    )?;
    Ok(Tag {
        id: tag_id,
        name: name.to_string(),
        color: color.to_string(),
    })
}

pub fn remove_tag_from_article(conn: &Connection, article_id: i64, tag_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM article_tags WHERE article_id = ?1 AND tag_id = ?2",
        params![article_id, tag_id],
    )?;
    Ok(())
}

pub fn delete_tag(conn: &Connection, tag_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM article_tags WHERE tag_id = ?1",
        params![tag_id],
    )?;
    conn.execute("DELETE FROM tags WHERE id = ?1", params![tag_id])?;
    Ok(())
}
