use crate::models::{Article, Feed, Folder};
use log::{debug, error, info, warn};
use rusqlite::{Connection, Result, params};
use rusqlite_migration::{M, Migrations};

pub fn init_db(conn: &mut Connection) -> std::result::Result<(), Box<dyn std::error::Error>> {
    info!("Initializing database schema");

    let migrations = Migrations::new(vec![
        M::up(
            "CREATE TABLE folders (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            );
            CREATE TABLE feeds (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL UNIQUE,
                folder_id INTEGER NOT NULL,
                FOREIGN KEY (folder_id) REFERENCES folders (id)
            );
            CREATE TABLE articles (
                id INTEGER PRIMARY KEY,
                feed_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                author TEXT,
                summary TEXT,
                url TEXT NOT NULL UNIQUE,
                timestamp INTEGER,
                FOREIGN KEY (feed_id) REFERENCES feeds (id)
            );",
        ),
        M::up("ALTER TABLE articles ADD COLUMN is_read BOOLEAN NOT NULL DEFAULT 0;"),
        M::up("ALTER TABLE articles ADD COLUMN is_saved BOOLEAN NOT NULL DEFAULT 0;"),
        // Migration 4: Add Error State to Feeds
        M::up("ALTER TABLE feeds ADD COLUMN has_error BOOLEAN NOT NULL DEFAULT 0;"),
    ]);

    match migrations.to_latest(conn) {
        Ok(_) => {
            info!("Database migrations applied successfully");
        }
        Err(e) => {
            error!("Failed to apply database migrations: {}", e);
            return Err(e.into());
        }
    }

    match conn.execute(
        "INSERT OR IGNORE INTO folders (id, name) VALUES (1, 'Uncategorized')",
        [],
    ) {
        Ok(_) => {
            debug!("Default 'Uncategorized' folder ensured");
        }
        Err(e) => {
            warn!("Failed to create default folder: {}", e);
        }
    }

    Ok(())
}

// --- Read Operations ---

pub fn get_folders_with_feeds(conn: &Connection) -> Result<Vec<Folder>> {
    debug!("Querying folders with feeds");

    let mut folder_stmt =
        conn.prepare("SELECT id, name FROM folders ORDER BY name COLLATE NOCASE")?;

    let mut feed_stmt = conn.prepare(
        "SELECT f.id, f.name, f.url, f.folder_id, f.has_error,
        (SELECT COUNT(*) FROM articles a WHERE a.feed_id = f.id AND a.is_read = 0) as unread_count
        FROM feeds f
        WHERE f.folder_id = ?1
        ORDER BY f.name COLLATE NOCASE",
    )?;

    let folders = folder_stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;

            let feeds = feed_stmt
                .query_map([id], |feed_row| {
                    Ok(Feed {
                        id: feed_row.get(0)?,
                        name: feed_row.get(1)?,
                        url: feed_row.get(2)?,
                        folder_id: feed_row.get(3)?,
                        has_error: feed_row.get::<_, bool>(4).unwrap_or(false),
                        unread_count: feed_row.get(5)?,
                    })
                })
                .and_then(|mapped_rows| mapped_rows.collect());

            Ok(Folder {
                id,
                name,
                feeds: feeds.unwrap_or_else(|_| vec![]),
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
         FROM articles
         WHERE feed_id = ?1
         ORDER BY timestamp {}
         LIMIT ?2 OFFSET ?3",
        order
    );

    let mut stmt = conn.prepare(&sql)?;
    map_articles(&mut stmt, params![feed_id, limit, offset])
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
         WHERE f.folder_id = ?1 AND a.is_read = 0
         ORDER BY a.timestamp {}
         LIMIT ?2 OFFSET ?3",
        order
    );

    let mut stmt = conn.prepare(&sql)?;
    map_articles(&mut stmt, params![folder_id, limit, offset])
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
         FROM articles
         WHERE timestamp > ?1
         ORDER BY timestamp {}
         LIMIT ?2 OFFSET ?3",
        order
    );

    let mut stmt = conn.prepare(&sql)?;
    map_articles(&mut stmt, params![cutoff_timestamp, limit, offset])
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
         FROM articles
         WHERE is_saved = 1
         ORDER BY timestamp {}
         LIMIT ?1 OFFSET ?2",
        order
    );

    let mut stmt = conn.prepare(&sql)?;
    map_articles(&mut stmt, params![limit, offset])
}

fn map_articles(
    stmt: &mut rusqlite::Statement,
    params: impl rusqlite::Params,
) -> Result<Vec<Article>> {
    let articles = stmt
        .query_map(params, |row| {
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
        .collect::<Result<Vec<Article>>>()?;
    Ok(articles)
}

pub fn get_feed_url(conn: &Connection, feed_id: i64) -> Result<String> {
    conn.query_row(
        "SELECT url FROM feeds WHERE id = ?1",
        params![feed_id],
        |row| row.get(0),
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
        |row| row.get(0),
    )
}

pub fn create_feed(conn: &Connection, name: &str, url: &str, folder_id: i64) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO feeds (name, url, folder_id, has_error) VALUES (?1, ?2, ?3, 0)",
        params![name, url, folder_id],
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

pub fn insert_article(conn: &Connection, article: &Article) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO articles (feed_id, title, author, summary, url, timestamp, is_read, is_saved)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0)",
        params![
            article.feed_id,
            article.title,
            article.author,
            article.summary,
            article.url,
            article.timestamp
        ],
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
        "UPDATE articles SET is_read = 1 WHERE feed_id = ?1",
        params![feed_id],
    )?;
    Ok(())
}

pub fn mark_folder_read(conn: &Connection, folder_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE articles SET is_read = 1 WHERE feed_id IN (SELECT id FROM feeds WHERE folder_id = ?1)",
        params![folder_id],
    )?;
    Ok(())
}

pub fn update_article_saved(conn: &Connection, article_id: i64, is_saved: bool) -> Result<()> {
    let val = if is_saved { 1 } else { 0 };
    conn.execute(
        "UPDATE articles SET is_saved = ?1 WHERE id = ?2",
        params![val, article_id],
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
