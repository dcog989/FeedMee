use log::info;
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
        M::up("CREATE INDEX IF NOT EXISTS idx_articles_feed_unread ON articles(feed_id, is_read);"),
        M::up(
            "DELETE FROM articles
             WHERE id NOT IN (
                 SELECT MIN(id) FROM articles
                 GROUP BY feed_id,
                     CASE
                         WHEN instr(url, '?access_token=') > 0 THEN substr(url, 1, instr(url, '?access_token=') - 1)
                         WHEN instr(url, '&access_token=') > 0 THEN substr(url, 1, instr(url, '&access_token=') - 1)
                         ELSE url
                     END
             );
             UPDATE articles SET url =
                 CASE
                     WHEN instr(url, '?access_token=') > 0 THEN substr(url, 1, instr(url, '?access_token=') - 1)
                     WHEN instr(url, '&access_token=') > 0 THEN substr(url, 1, instr(url, '&access_token=') - 1)
                     ELSE url
                 END
             WHERE instr(url, '?access_token=') > 0 OR instr(url, '&access_token=') > 0;",
        ),
        M::up("ALTER TABLE feeds ADD COLUMN error_count INTEGER NOT NULL DEFAULT 0;"),
        M::up(
            "ALTER TABLE articles ADD COLUMN created_at INTEGER NOT NULL DEFAULT 0;
             UPDATE articles SET created_at = CASE
                 WHEN timestamp > 0 THEN timestamp
                 ELSE CAST(strftime('%s','now') AS INTEGER)
             END;",
        ),
        M::up("CREATE INDEX IF NOT EXISTS idx_articles_saved ON articles(timestamp, id) WHERE is_saved = 1;"),
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
