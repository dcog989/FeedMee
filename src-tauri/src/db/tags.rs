use crate::models::Tag;
use rusqlite::{Connection, Result, params};

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

pub fn add_tag_to_article(conn: &Connection, article_id: i64, name: &str, color: &str) -> Result<Tag> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT OR IGNORE INTO tags (name, color) VALUES (?1, ?2)",
        params![name, color],
    )?;
    let tag_id: i64 = tx.query_row("SELECT id FROM tags WHERE name = ?1", params![name], |r| r.get(0))?;
    tx.execute(
        "INSERT OR IGNORE INTO article_tags (article_id, tag_id) VALUES (?1, ?2)",
        params![article_id, tag_id],
    )?;
    tx.commit()?;
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
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM article_tags WHERE tag_id = ?1", params![tag_id])?;
    tx.execute("DELETE FROM tags WHERE id = ?1", params![tag_id])?;
    tx.commit()
}
