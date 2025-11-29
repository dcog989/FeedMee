use crate::models::{Article, Feed, Folder};
use rusqlite::{Connection, Result, params};
use rusqlite_migration::{M, Migrations};

// Changed return type to Box<dyn std::error::Error> to handle mixed error types
pub fn init_db(conn: &mut Connection) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // 1. Define the schema evolution
    let migrations = Migrations::new(vec![M::up(
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
    )]);

    // 2. Apply migrations
    migrations.to_latest(conn)?;

    Ok(())
}

pub fn get_folders_with_feeds(conn: &Connection) -> Result<Vec<Folder>> {
    let mut folder_stmt = conn.prepare("SELECT id, name FROM folders ORDER BY name")?;
    let mut feed_stmt = conn
        .prepare("SELECT id, name, url, folder_id FROM feeds WHERE folder_id = ?1 ORDER BY name")?;

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
) -> Result<Vec<Article>> {
    let mut stmt = conn.prepare(
        "SELECT id, feed_id, title, author, summary, url, timestamp
         FROM articles
         WHERE feed_id = ?1
         ORDER BY timestamp DESC
         LIMIT ?2 OFFSET ?3",
    )?;

    let articles = stmt
        .query_map(params![feed_id, limit, offset], |row| {
            Ok(Article {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                title: row.get(2)?,
                author: row.get(3).unwrap_or_default(),
                summary: row.get(4).unwrap_or_default(),
                url: row.get(5)?,
                timestamp: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<Article>>>()?;
    Ok(articles)
}

// --- Write Operations ---

pub fn create_folder(conn: &Connection, name: &str) -> Result<i64> {
    // Try to insert, ignore if exists, then fetch ID
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
        "INSERT OR IGNORE INTO feeds (name, url, folder_id) VALUES (?1, ?2, ?3)",
        params![name, url, folder_id],
    )?;
    Ok(())
}

pub fn get_feed_url(conn: &Connection, feed_id: i64) -> Result<String> {
    conn.query_row(
        "SELECT url FROM feeds WHERE id = ?1",
        params![feed_id],
        |row| row.get(0),
    )
}

pub fn insert_article(conn: &Connection, article: &Article) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO articles (feed_id, title, author, summary, url, timestamp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
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
