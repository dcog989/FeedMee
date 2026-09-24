pub mod articles;
pub mod feeds;
pub mod schema;
pub mod tags;

pub use articles::*;
pub use feeds::*;
pub use schema::*;
pub use tags::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Article;
    use rusqlite::Connection;

    fn test_conn() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        init_db(&mut conn).unwrap();
        conn
    }

    fn sample_article(feed_id: i64) -> Article {
        Article {
            id: 0,
            feed_id,
            title: "Title".to_string(),
            author: "Author".to_string(),
            summary: "Summary".to_string(),
            url: "https://example.com/article".to_string(),
            image_url: "https://example.com/image.jpg".to_string(),
            image_low_res: false,
            timestamp: 1000,
            is_read: false,
            is_saved: false,
            has_tags: false,
        }
    }

    #[test]
    fn article_projections_map_all_columns() {
        let mut conn = test_conn();
        let folder = create_folder(&conn, "News").unwrap();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", Some(folder), "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)]).unwrap();

        let id = get_articles_for_feed(&conn, feed, 1, 0, true).unwrap()[0].id;
        update_article_saved(&conn, id, true).unwrap();

        let result_sets = [
            get_articles_for_feed(&conn, feed, 10, 0, true).unwrap(),
            get_articles_for_folder(&conn, folder, 10, 0, true).unwrap(),
            get_latest_articles(&conn, 0, 10, 0, true).unwrap(),
            get_saved_articles(&conn, 10, 0, true).unwrap(),
            search_articles(&conn, "Title", 10, 0, false).unwrap(),
        ];

        for articles in result_sets {
            assert_eq!(articles.len(), 1);
            let a = &articles[0];
            assert_eq!(a.title, "Title");
            assert_eq!(a.author, "Author");
            assert_eq!(a.summary, "Summary");
            assert_eq!(a.url, "https://example.com/article");
            assert_eq!(a.image_url, "https://example.com/image.jpg");
            assert_eq!(a.timestamp, 1000);
            assert!(a.is_saved);
            assert!(!a.has_tags);
        }
    }

    #[test]
    fn feed_projection_maps_fields_and_groups_by_folder() {
        let conn = test_conn();
        let folder = create_folder(&conn, "News").unwrap();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", Some(folder), "rss").unwrap();

        let single = get_feed(&conn, feed).unwrap();
        assert_eq!(single.name, "Feed");
        assert_eq!(single.url, "https://example.com/rss");
        assert_eq!(single.folder_id, Some(folder));
        assert_eq!(single.display_url, "https://example.com/rss");
        assert_eq!(single.unread_count, 0);

        let folders = get_folders_with_feeds(&conn).unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].id, folder);
        assert_eq!(folders[0].feeds.len(), 1);
        assert_eq!(folders[0].feeds[0].name, "Feed");
    }

    #[test]
    fn purge_reclaims_undated_articles_by_created_at() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();

        let mut undated = sample_article(feed);
        undated.timestamp = 0;
        undated.url = "https://example.com/undated".to_string();
        batch_insert_articles(&mut conn, &[undated]).unwrap();

        assert_eq!(purge_old_articles(&conn, 90).unwrap(), 0);

        conn.execute("UPDATE articles SET created_at = 1", []).unwrap();
        assert_eq!(purge_old_articles(&conn, 90).unwrap(), 1);
    }

    #[test]
    fn delete_folder_removes_nested_feeds_and_articles() {
        let mut conn = test_conn();
        let folder = create_folder(&conn, "News").unwrap();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", Some(folder), "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)]).unwrap();

        delete_folder(&conn, folder).unwrap();

        assert!(get_folders_with_feeds(&conn).unwrap().is_empty());
        assert!(get_articles_for_feed(&conn, feed, 10, 0, true).unwrap().is_empty());
        assert!(get_feed(&conn, feed).is_err());
    }
}
