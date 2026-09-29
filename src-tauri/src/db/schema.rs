use log::info;
use rusqlite::{Connection, Result, params};
use rusqlite_migration::{M, Migrations};

pub const DB_FILENAME: &str = "feedmee.sqlite";

/// Hidden feed that retains saved articles after their original feed is
/// deleted. Never listed to the user; mirrors the `folder_id = 0` sentinel.
pub const SAVED_FEED_ID: i64 = 0;

/// Consolidated baseline schema. Earlier incremental migrations were collapsed
/// into this single migration; fresh installs only. Existing databases must be
/// recreated (the app does not upgrade pre-baseline DBs).
fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(
        "CREATE TABLE IF NOT EXISTS folders (
            id   INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        );
        CREATE TABLE IF NOT EXISTS feeds (
            id             INTEGER PRIMARY KEY,
            name           TEXT NOT NULL,
            url            TEXT NOT NULL,
            folder_id      INTEGER NOT NULL,
            has_error      BOOLEAN NOT NULL DEFAULT 0,
            feed_type      TEXT NOT NULL DEFAULT 'rss',
            bluesky_cursor TEXT,
            error_count    INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (folder_id) REFERENCES folders (id)
        );
        CREATE TABLE IF NOT EXISTS articles (
            id         INTEGER PRIMARY KEY,
            feed_id    INTEGER NOT NULL,
            title      TEXT NOT NULL,
            author     TEXT,
            summary    TEXT,
            url        TEXT NOT NULL,
            timestamp  INTEGER,
            is_read    BOOLEAN NOT NULL DEFAULT 0,
            is_saved   BOOLEAN NOT NULL DEFAULT 0,
            image_url  TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL DEFAULT 0,
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
        CREATE TRIGGER IF NOT EXISTS articles_ai AFTER INSERT ON articles BEGIN
            INSERT INTO articles_fts(rowid, title, author, summary)
                VALUES (new.id, new.title, COALESCE(new.author,''), COALESCE(new.summary,''));
        END;
        CREATE TRIGGER IF NOT EXISTS articles_ad AFTER DELETE ON articles BEGIN
            INSERT INTO articles_fts(articles_fts, rowid, title, author, summary)
                VALUES ('delete', old.id, old.title, COALESCE(old.author,''), COALESCE(old.summary,''));
        END;
        CREATE TRIGGER IF NOT EXISTS articles_au AFTER UPDATE ON articles
        WHEN old.title IS NOT new.title OR old.author IS NOT new.author OR old.summary IS NOT new.summary
        BEGIN
            INSERT INTO articles_fts(articles_fts, rowid, title, author, summary)
                VALUES ('delete', old.id, old.title, COALESCE(old.author,''), COALESCE(old.summary,''));
            INSERT INTO articles_fts(rowid, title, author, summary)
                VALUES (new.id, new.title, COALESCE(new.author,''), COALESCE(new.summary,''));
        END;
        CREATE INDEX IF NOT EXISTS idx_articles_feed_timestamp ON articles(feed_id, timestamp);
        CREATE INDEX IF NOT EXISTS idx_articles_timestamp ON articles(timestamp);
        CREATE INDEX IF NOT EXISTS idx_articles_feed_unread ON articles(feed_id, is_read);
        CREATE INDEX IF NOT EXISTS idx_articles_saved ON articles(timestamp, id) WHERE is_saved = 1;
        INSERT OR IGNORE INTO folders (id, name) VALUES (0, '');
        INSERT OR IGNORE INTO feeds (id, name, url, folder_id, has_error, feed_type)
            VALUES (0, 'Saved', 'feedmee:saved', 0, 0, 'rss');",
    )])
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
    if retention_days == 0 {
        info!("Article retention disabled; skipping purge");
        return Ok(0);
    }
    let cutoff = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        - (retention_days as i64 * 86400);
    let count = conn.execute(
        "DELETE FROM articles
         WHERE COALESCE(NULLIF(timestamp, 0), created_at) < ?1
           AND is_saved = 0
           AND NOT EXISTS (SELECT 1 FROM article_tags WHERE article_id = articles.id)",
        params![cutoff],
    )?;
    if count > 0 {
        info!("Purged {} old articles (retention: {} days)", count, retention_days);
    }
    Ok(count)
}
