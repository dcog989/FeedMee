use super::SAVED_FEED_ID;
use crate::models::{Feed, Folder};
use log::debug;
use rusqlite::{Connection, OptionalExtension, Result, params};
use std::collections::HashMap;

fn feed_derived_fields(url: &str, feed_type: &str) -> (String, String) {
    if feed_type == "bluesky" {
        let did = url.trim_prefix("bsky:");
        (String::new(), did.to_string())
    } else {
        (url.to_string(), url.to_string())
    }
}

/// Shared projection for feed queries. `map_feed_row` reads columns by
/// positional index, so every query must select exactly this list in order.
const FEED_SELECT: &str = "SELECT f.id, f.name, f.url, f.folder_id, f.has_error, f.feed_type, \
     COALESCE(uc.unread_count, 0) AS unread_count, f.error_count \
     FROM feeds f \
     LEFT JOIN ( \
         SELECT feed_id, COUNT(*) AS unread_count FROM articles WHERE is_read = 0 GROUP BY feed_id \
     ) uc ON f.id = uc.feed_id";

fn map_feed_row(r: &rusqlite::Row) -> rusqlite::Result<Feed> {
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
        error_count: r.get(7)?,
        display_url,
        source_id,
    })
}

pub fn get_folders_with_feeds(conn: &Connection) -> Result<Vec<Folder>> {
    debug!("Querying folders with feeds");

    // One execution for every feed: the unread aggregate inside FEED_SELECT
    // scans the whole articles table, so running it per folder is F full scans.
    let mut feed_stmt = conn.prepare(&format!(
        "{FEED_SELECT} WHERE f.id != {SAVED_FEED_ID} ORDER BY f.name COLLATE NOCASE"
    ))?;

    let mut feeds_by_folder: HashMap<i64, Vec<Feed>> = HashMap::new();
    for feed in feed_stmt.query_map([], map_feed_row)? {
        let feed = feed?;
        feeds_by_folder
            .entry(feed.folder_id.unwrap_or(0))
            .or_default()
            .push(feed);
    }

    let mut folder_stmt = conn.prepare("SELECT id, name FROM folders WHERE id != 0 ORDER BY name COLLATE NOCASE")?;
    let mut folders: Vec<Folder> = folder_stmt
        .query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                feeds: Vec::new(),
            })
        })?
        .collect::<Result<Vec<Folder>>>()?;

    for folder in &mut folders {
        if let Some(feeds) = feeds_by_folder.remove(&folder.id) {
            folder.feeds = feeds;
        }
    }

    if let Some(root_feeds) = feeds_by_folder.remove(&0) {
        folders.push(Folder {
            id: 0,
            name: String::new(),
            feeds: root_feeds,
        });
    }

    Ok(folders)
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
        &format!("{FEED_SELECT} WHERE f.id = ?1"),
        params![feed_id],
        map_feed_row,
    )
}

pub fn create_folder(conn: &Connection, name: &str) -> Result<i64> {
    // Case-insensitive so "News" and "news" cannot become two folders.
    if let Some(id) = conn
        .query_row(
            "SELECT id FROM folders WHERE name = ?1 COLLATE NOCASE",
            params![name],
            |r| r.get(0),
        )
        .optional()?
    {
        return Ok(id);
    }
    conn.execute("INSERT INTO folders (name) VALUES (?1)", params![name])?;
    Ok(conn.last_insert_rowid())
}

pub fn feed_exists_by_url(conn: &Connection, url: &str, exclude_id: Option<i64>) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM feeds WHERE url = ?1 AND (?2 IS NULL OR id != ?2))",
        params![url, exclude_id],
        |r| r.get(0),
    )
}

/// A folder name is unique regardless of case, so renaming cannot create a
/// second "News" next to the existing one.
pub fn folder_name_exists(conn: &Connection, name: &str, exclude_id: Option<i64>) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM folders WHERE name = ?1 COLLATE NOCASE AND (?2 IS NULL OR id != ?2))",
        params![name, exclude_id],
        |r| r.get(0),
    )
}

pub fn create_feed(conn: &Connection, name: &str, url: &str, folder_id: Option<i64>, feed_type: &str) -> Result<i64> {
    let fid = folder_id.unwrap_or(0);
    conn.execute(
        "INSERT INTO feeds (name, url, folder_id, has_error, feed_type) VALUES (?1, ?2, ?3, 0, ?4)",
        params![name, url, fid, feed_type],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn record_refresh_success(conn: &Connection, feed_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE feeds SET has_error = 0, error_count = 0 WHERE id = ?1",
        params![feed_id],
    )?;
    Ok(())
}

pub fn record_refresh_failure(conn: &Connection, feed_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE feeds SET has_error = 1, error_count = error_count + 1 WHERE id = ?1",
        params![feed_id],
    )?;
    Ok(())
}

pub fn get_bluesky_cursor(conn: &Connection, feed_id: i64) -> Result<Option<String>> {
    // A missing row, NULL cursor and empty cursor all mean "start fresh";
    // anything else is a real error and must not be mistaken for one of them.
    conn.query_row(
        "SELECT bluesky_cursor FROM feeds WHERE id = ?1",
        params![feed_id],
        |row| row.get::<_, Option<String>>(0),
    )
    .optional()
    .map(|cursor| cursor.flatten().filter(|c| !c.is_empty()))
}

pub fn set_bluesky_cursor(conn: &Connection, feed_id: i64, cursor: &str) -> Result<()> {
    conn.execute(
        "UPDATE feeds SET bluesky_cursor = ?1 WHERE id = ?2",
        params![cursor, feed_id],
    )?;
    Ok(())
}

pub fn rename_folder(conn: &Connection, id: i64, new_name: &str) -> Result<()> {
    conn.execute("UPDATE folders SET name = ?1 WHERE id = ?2", params![new_name, id])?;
    Ok(())
}

pub fn rename_feed(conn: &Connection, id: i64, new_name: &str, new_url: &str) -> Result<()> {
    conn.execute(
        "UPDATE feeds SET name = ?1, url = ?2 WHERE id = ?3",
        params![new_name, new_url, id],
    )?;
    Ok(())
}

/// An article the user asked to keep. Retention and "mark all read" treat
/// saved and tagged rows as keep signals, so deleting a feed must too.
fn kept_articles(table: &str) -> String {
    format!("{table}.is_saved = 1 OR EXISTS (SELECT 1 FROM article_tags WHERE article_id = {table}.id)")
}

fn delete_feed_rows(conn: &Connection, id: i64) -> Result<()> {
    if id == SAVED_FEED_ID {
        return Ok(());
    }
    // Tags on the copy leaving the feed must land on the hidden copy that stays
    // behind, or the tag links cascade away with the deleted row.
    conn.execute(
        &format!(
            "INSERT OR IGNORE INTO article_tags (article_id, tag_id)
             SELECT hidden.id, at.tag_id
             FROM articles kept
             JOIN articles hidden ON hidden.feed_id = ?2 AND hidden.url = kept.url
             JOIN article_tags at ON at.article_id = kept.id
             WHERE kept.feed_id = ?1 AND ({})",
            kept_articles("kept")
        ),
        params![id, SAVED_FEED_ID],
    )?;
    // The hidden copy survives the merge, so carry over state applied to the
    // incoming copy; otherwise the article would drop out of Read Later or
    // reappear as unread.
    conn.execute(
        "UPDATE articles SET is_saved = 1
         WHERE feed_id = ?2
           AND url IN (SELECT kept.url FROM articles kept WHERE kept.feed_id = ?1 AND kept.is_saved = 1)",
        params![id, SAVED_FEED_ID],
    )?;
    conn.execute(
        "UPDATE articles SET is_read = 1
         WHERE feed_id = ?2
           AND url IN (SELECT kept.url FROM articles kept WHERE kept.feed_id = ?1 AND kept.is_read = 1)",
        params![id, SAVED_FEED_ID],
    )?;
    // The hidden copy already holds this URL; keep it so its saved and read
    // state survive, and drop the incoming duplicate.
    conn.execute(
        &format!(
            "DELETE FROM articles
             WHERE feed_id = ?1 AND ({})
               AND url IN (SELECT hidden.url FROM articles hidden WHERE hidden.feed_id = ?2)",
            kept_articles("articles")
        ),
        params![id, SAVED_FEED_ID],
    )?;
    conn.execute(
        &format!(
            "UPDATE articles SET feed_id = ?2 WHERE feed_id = ?1 AND ({})",
            kept_articles("articles")
        ),
        params![id, SAVED_FEED_ID],
    )?;
    conn.execute("DELETE FROM articles WHERE feed_id = ?1", params![id])?;
    conn.execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn delete_feed(conn: &Connection, id: i64) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    delete_feed_rows(&tx, id)?;
    tx.commit()
}

pub fn delete_folder(conn: &Connection, id: i64) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    let feed_ids: Vec<i64> = {
        let mut stmt = tx.prepare("SELECT id FROM feeds WHERE folder_id = ?1")?;
        stmt.query_map(params![id], |row| row.get(0))?
            .collect::<Result<Vec<i64>>>()?
    };
    for feed_id in feed_ids {
        delete_feed_rows(&tx, feed_id)?;
    }
    tx.execute("DELETE FROM folders WHERE id = ?1", params![id])?;
    tx.commit()
}

pub fn move_feed(conn: &Connection, feed_id: i64, target_folder_id: Option<i64>) -> Result<()> {
    let fid = target_folder_id.unwrap_or(0);
    conn.execute("UPDATE feeds SET folder_id = ?1 WHERE id = ?2", params![fid, feed_id])?;
    Ok(())
}
