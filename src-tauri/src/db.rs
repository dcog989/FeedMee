use crate::models::{Article, Feed, Folder};
use rusqlite::{Connection, Result};
use rusqlite_migration::{M, Migrations};

// Changed return type to Box<dyn std::error::Error> to handle mixed error types
pub fn init_db(conn: &mut Connection) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // 1. Define the schema evolution
    let migrations = Migrations::new(vec![M::up(
        "CREATE TABLE folders (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
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
    // The '?' operator now works because both error types implement std::error::Error
    migrations.to_latest(conn)?;

    // 3. Seed dummy data
    seed_demo_data(conn)?;

    Ok(())
}

fn seed_demo_data(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM folders", [], |row| row.get(0))?;
    if count == 0 {
        conn.execute("INSERT INTO folders (id, name) VALUES (1, 'Tech News')", ())?;
        conn.execute("INSERT INTO folders (id, name) VALUES (2, 'Design')", ())?;

        conn.execute(
            "INSERT INTO feeds (id, name, url, folder_id) VALUES (1, 'Ars Technica', 'https://arstechnica.com/rss', 1)",
            (),
        )?;
        conn.execute(
            "INSERT INTO feeds (id, name, url, folder_id) VALUES (2, 'Hacker News', 'https://news.ycombinator.com/rss', 1)",
            (),
        )?;
        conn.execute(
            "INSERT INTO feeds (id, name, url, folder_id) VALUES (3, 'A List Apart', 'https://alistapart.com/main/feed/', 2)",
            (),
        )?;

        conn.execute(
            "INSERT INTO articles (feed_id, title, author, summary, url, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (1, "First Article from Ars", "John Doe", "This is a summary of the first article.", "https://example.com/1", 1700000000),
        )?;
        conn.execute(
            "INSERT INTO articles (feed_id, title, author, summary, url, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (1, "Second Article from Ars", "Jane Smith", "Summary of the second post.", "https://example.com/2", 1700000100),
        )?;
        conn.execute(
            "INSERT INTO articles (feed_id, title, author, summary, url, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (3, "A Post about Design", "Designer Dave", "<p>Exploring modern design principles.</p>", "https://example.com/3", 1700000200),
        )?;
    }
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

pub fn get_articles_for_feed(conn: &Connection, feed_id: i64) -> Result<Vec<Article>> {
    let mut stmt = conn.prepare("SELECT id, feed_id, title, author, summary, url, timestamp FROM articles WHERE feed_id = ?1 ORDER BY timestamp DESC")?;
    let articles = stmt
        .query_map([feed_id], |row| {
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
