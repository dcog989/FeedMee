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
    fn init_db_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        init_db(&mut conn).unwrap();
        init_db(&mut conn).unwrap();
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn init_db_refuses_a_pre_baseline_schema() {
        let mut conn = test_conn();
        // Pre-baseline databases carried a higher counter and lack columns the
        // baseline adds, so they must be moved aside, never re-stamped.
        conn.pragma_update(None, "user_version", 12).unwrap();

        let err = init_db(&mut conn).unwrap_err();
        assert!(err.downcast_ref::<UnsupportedSchema>().is_some());
    }

    #[test]
    fn article_projections_map_all_columns() {
        let mut conn = test_conn();
        let folder = create_folder(&conn, "News").unwrap();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", Some(folder), "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)], None).unwrap();

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
    fn purge_leaves_undated_articles_alone() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();

        let mut undated = sample_article(feed);
        undated.timestamp = 0;
        undated.url = "https://example.com/undated".to_string();
        batch_insert_articles(&mut conn, &[undated], None).unwrap();

        conn.execute("UPDATE articles SET created_at = 1", []).unwrap();
        assert_eq!(purge_old_articles(&conn, 90).unwrap(), 0);
        assert_eq!(get_articles_for_feed(&conn, feed, 10, 0, true).unwrap().len(), 1);
    }

    #[test]
    fn duplicate_articles_upgrade_image_and_summary_without_duplicating() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();

        let mut first = sample_article(feed);
        first.image_url = "https://example.com/low.jpg".to_string();
        first.image_low_res = true;
        first.summary = "Old".to_string();
        assert_eq!(batch_insert_articles(&mut conn, &[first], None).unwrap(), 1);

        let mut second = sample_article(feed);
        second.image_url = "https://example.com/high.jpg".to_string();
        second.summary = "New".to_string();
        assert_eq!(batch_insert_articles(&mut conn, &[second], None).unwrap(), 0);

        let mut third = sample_article(feed);
        third.image_url = "https://example.com/other-low.jpg".to_string();
        third.image_low_res = true;
        batch_insert_articles(&mut conn, &[third], None).unwrap();

        let articles = get_articles_for_feed(&conn, feed, 10, 0, true).unwrap();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].image_url, "https://example.com/high.jpg");
        assert_eq!(articles[0].summary, "New");
    }

    #[test]
    fn bulk_mark_read_spares_saved_and_tagged_articles() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();

        let mut plain = sample_article(feed);
        plain.url = "https://example.com/plain".to_string();
        let mut saved = sample_article(feed);
        saved.url = "https://example.com/saved".to_string();
        let mut tagged = sample_article(feed);
        tagged.url = "https://example.com/tagged".to_string();
        batch_insert_articles(&mut conn, &[plain, saved, tagged], None).unwrap();

        let id_for = |url: &str| {
            get_articles_for_feed(&conn, feed, 10, 0, true)
                .unwrap()
                .into_iter()
                .find(|a| a.url == url)
                .unwrap()
                .id
        };
        update_article_saved(&conn, id_for("https://example.com/saved"), true).unwrap();
        add_tag_to_article(&conn, id_for("https://example.com/tagged"), "Tag", "#4899ec").unwrap();

        mark_feed_read(&conn, feed).unwrap();

        let read_state = |url: &str| {
            get_articles_for_feed(&conn, feed, 10, 0, true)
                .unwrap()
                .into_iter()
                .find(|a| a.url == url)
                .unwrap()
                .is_read
        };
        assert!(read_state("https://example.com/plain"));
        assert!(!read_state("https://example.com/saved"));
        assert!(!read_state("https://example.com/tagged"));
    }

    #[test]
    fn unsaving_a_sentinel_article_removes_the_orphan() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)], None).unwrap();

        let id = get_articles_for_feed(&conn, feed, 10, 0, true).unwrap()[0].id;
        update_article_saved(&conn, id, true).unwrap();
        delete_feed(&conn, feed).unwrap();

        update_article_saved(&conn, id, false).unwrap();

        assert!(get_saved_articles(&conn, 10, 0, true).unwrap().is_empty());
        let remaining: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM articles WHERE id = ?1",
                rusqlite::params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(remaining, 0);
    }

    #[test]
    fn folder_feeds_report_unread_counts() {
        let mut conn = test_conn();
        let folder = create_folder(&conn, "News").unwrap();
        let busy = create_feed(&conn, "Busy", "https://example.com/busy", Some(folder), "rss").unwrap();
        let quiet = create_feed(&conn, "Quiet", "https://example.com/quiet", Some(folder), "rss").unwrap();
        let root_feed = create_feed(&conn, "Root", "https://example.com/root", None, "rss").unwrap();

        let mut first = sample_article(busy);
        first.url = "https://example.com/one".to_string();
        let mut second = sample_article(busy);
        second.url = "https://example.com/two".to_string();
        let mut third = sample_article(quiet);
        third.url = "https://example.com/three".to_string();
        let mut fourth = sample_article(root_feed);
        fourth.url = "https://example.com/four".to_string();
        batch_insert_articles(&mut conn, &[first, second, third, fourth], None).unwrap();

        let read_id = get_articles_for_feed(&conn, busy, 10, 0, true).unwrap()[0].id;
        set_article_read(&conn, read_id, true).unwrap();

        let folders = get_folders_with_feeds(&conn).unwrap();
        assert_eq!(folders.len(), 2);

        let news = folders.iter().find(|f| f.id == folder).unwrap();
        assert_eq!(news.feeds.len(), 2);
        assert_eq!(news.feeds.iter().find(|f| f.id == busy).unwrap().unread_count, 1);
        assert_eq!(news.feeds.iter().find(|f| f.id == quiet).unwrap().unread_count, 1);

        let root = folders.iter().find(|f| f.id == 0).unwrap();
        assert_eq!(root.feeds.len(), 1);
        assert_eq!(root.feeds[0].id, root_feed);
        assert_eq!(root.feeds[0].unread_count, 1);

        assert_eq!(get_feed_unread_count(&conn, busy).unwrap(), 1);
    }

    #[test]
    fn folder_names_are_unique_regardless_of_case() {
        let conn = test_conn();
        let news = create_folder(&conn, "News").unwrap();
        let other = create_folder(&conn, "Tech").unwrap();

        assert!(folder_name_exists(&conn, "news", None).unwrap());
        assert!(!folder_name_exists(&conn, "news", Some(news)).unwrap());
        assert!(folder_name_exists(&conn, "news", Some(other)).unwrap());
        assert!(!folder_name_exists(&conn, "Podcasts", None).unwrap());
    }

    #[test]
    fn adding_an_existing_tag_reuses_its_id_and_colour() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)], None).unwrap();
        let article_id = get_articles_for_feed(&conn, feed, 10, 0, true).unwrap()[0].id;

        let first = add_tag_to_article(&conn, article_id, "News", "#ff0000").unwrap();
        let second = add_tag_to_article(&conn, article_id, "news", "#0000ff").unwrap();

        assert_eq!(second.id, first.id);
        assert_eq!(second.name, "News");
        assert_eq!(second.color, "#ff0000");
        assert_eq!(get_tags_for_article(&conn, article_id).unwrap().len(), 1);
    }

    #[test]
    fn expired_articles_are_not_reinserted_on_refresh() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();

        let cutoff = 1_000_000;

        let mut expired = sample_article(feed);
        expired.timestamp = cutoff - 1;
        expired.url = "https://example.com/expired".to_string();

        let mut current = sample_article(feed);
        current.timestamp = cutoff + 1;
        current.url = "https://example.com/current".to_string();

        let mut undated = sample_article(feed);
        undated.timestamp = 0;
        undated.url = "https://example.com/undated".to_string();

        let inserted = batch_insert_articles(&mut conn, &[expired, current, undated], Some(cutoff)).unwrap();

        assert_eq!(inserted, 2);
        let urls: Vec<String> = get_articles_for_feed(&conn, feed, 10, 0, true)
            .unwrap()
            .into_iter()
            .map(|a| a.url)
            .collect();
        assert!(!urls.contains(&"https://example.com/expired".to_string()));
        assert!(urls.contains(&"https://example.com/current".to_string()));
        assert!(urls.contains(&"https://example.com/undated".to_string()));
    }

    #[test]
    fn retention_cutoff_is_none_when_disabled() {
        assert_eq!(retention_cutoff(0), None);
        assert!(retention_cutoff(30).is_some());
    }

    #[test]
    fn purge_disabled_when_retention_zero() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)], None).unwrap();
        conn.execute("UPDATE articles SET created_at = 1", []).unwrap();

        assert_eq!(purge_old_articles(&conn, 0).unwrap(), 0);
        assert_eq!(get_articles_for_feed(&conn, feed, 10, 0, true).unwrap().len(), 1);
    }

    #[test]
    fn delete_feed_preserves_saved_articles() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();

        let mut saved = sample_article(feed);
        saved.url = "https://example.com/saved".to_string();
        let mut unsaved = sample_article(feed);
        unsaved.url = "https://example.com/unsaved".to_string();
        batch_insert_articles(&mut conn, &[saved, unsaved], None).unwrap();

        let saved_id = get_articles_for_feed(&conn, feed, 10, 0, true)
            .unwrap()
            .into_iter()
            .find(|a| a.url == "https://example.com/saved")
            .unwrap()
            .id;
        update_article_saved(&conn, saved_id, true).unwrap();

        delete_feed(&conn, feed).unwrap();

        assert!(get_feed(&conn, feed).is_err());
        assert!(get_articles_for_feed(&conn, feed, 10, 0, true).unwrap().is_empty());
        let remaining = get_saved_articles(&conn, 10, 0, true).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, saved_id);
    }

    #[test]
    fn sentinel_feed_is_hidden_from_listings() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)], None).unwrap();

        let folders = get_folders_with_feeds(&conn).unwrap();
        let ids: Vec<i64> = folders.iter().flat_map(|f| f.feeds.iter().map(|fe| fe.id)).collect();
        assert_eq!(ids, vec![feed]);
    }

    #[test]
    fn delete_folder_removes_nested_feeds_and_articles() {
        let mut conn = test_conn();
        let folder = create_folder(&conn, "News").unwrap();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", Some(folder), "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)], None).unwrap();

        delete_folder(&conn, folder).unwrap();

        assert!(get_folders_with_feeds(&conn).unwrap().is_empty());
        assert!(get_articles_for_feed(&conn, feed, 10, 0, true).unwrap().is_empty());
        assert!(get_feed(&conn, feed).is_err());
    }

    #[test]
    fn delete_folder_preserves_saved_articles() {
        let mut conn = test_conn();
        let folder = create_folder(&conn, "News").unwrap();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", Some(folder), "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)], None).unwrap();
        let id = get_articles_for_feed(&conn, feed, 10, 0, true).unwrap()[0].id;
        update_article_saved(&conn, id, true).unwrap();

        delete_folder(&conn, folder).unwrap();

        let saved = get_saved_articles(&conn, 10, 0, true).unwrap();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].id, id);
    }

    #[test]
    fn deleting_a_readded_feed_keeps_tags_on_the_surviving_copy() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)], None).unwrap();
        let id = get_articles_for_feed(&conn, feed, 10, 0, true).unwrap()[0].id;
        update_article_saved(&conn, id, true).unwrap();
        add_tag_to_article(&conn, id, "Keep", "#4899ec").unwrap();
        delete_feed(&conn, feed).unwrap();

        // Add the same feed and article again, save it, and delete it again.
        let readded = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(readded)], None).unwrap();
        let new_id = get_articles_for_feed(&conn, readded, 10, 0, true).unwrap()[0].id;
        update_article_saved(&conn, new_id, true).unwrap();
        delete_feed(&conn, readded).unwrap();

        let saved = get_saved_articles(&conn, 10, 0, true).unwrap();
        assert_eq!(saved.len(), 1);
        let tags = get_tags_for_article(&conn, saved[0].id).unwrap();
        assert_eq!(tags.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(), vec!["Keep"]);
    }

    #[test]
    fn deleting_a_readded_feed_carries_a_new_tag_to_the_surviving_copy() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)], None).unwrap();
        let id = get_articles_for_feed(&conn, feed, 10, 0, true).unwrap()[0].id;
        update_article_saved(&conn, id, true).unwrap();
        delete_feed(&conn, feed).unwrap();

        // Re-add the feed, tag the new copy, and delete again without saving it.
        let readded = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(readded)], None).unwrap();
        let new_id = get_articles_for_feed(&conn, readded, 10, 0, true).unwrap()[0].id;
        add_tag_to_article(&conn, new_id, "Keep", "#4899ec").unwrap();
        delete_feed(&conn, readded).unwrap();

        let saved = get_saved_articles(&conn, 10, 0, true).unwrap();
        assert_eq!(saved.len(), 1);
        let tags = get_tags_for_article(&conn, saved[0].id).unwrap();
        assert_eq!(tags.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(), vec!["Keep"]);
    }

    #[test]
    fn deleting_a_readded_feed_keeps_the_read_state_of_the_new_copy() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)], None).unwrap();
        let id = get_articles_for_feed(&conn, feed, 10, 0, true).unwrap()[0].id;
        update_article_saved(&conn, id, true).unwrap();
        delete_feed(&conn, feed).unwrap();

        // Read the article again in a re-added feed, then delete that feed.
        let readded = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(readded)], None).unwrap();
        let new_id = get_articles_for_feed(&conn, readded, 10, 0, true).unwrap()[0].id;
        set_article_read(&conn, new_id, true).unwrap();
        delete_feed(&conn, readded).unwrap();

        let saved = get_saved_articles(&conn, 10, 0, true).unwrap();
        assert_eq!(saved.len(), 1);
        assert!(saved[0].is_read);
    }

    #[test]
    fn deleting_a_readded_feed_keeps_a_newly_saved_article() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)], None).unwrap();
        let id = get_articles_for_feed(&conn, feed, 10, 0, true).unwrap()[0].id;
        add_tag_to_article(&conn, id, "Keep", "#4899ec").unwrap();
        delete_feed(&conn, feed).unwrap();

        // The hidden copy is tagged but not saved; save it after re-adding the feed.
        let readded = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(readded)], None).unwrap();
        let new_id = get_articles_for_feed(&conn, readded, 10, 0, true).unwrap()[0].id;
        update_article_saved(&conn, new_id, true).unwrap();
        delete_feed(&conn, readded).unwrap();

        assert_eq!(get_saved_articles(&conn, 10, 0, true).unwrap().len(), 1);
    }

    #[test]
    fn deleting_a_feed_keeps_tagged_articles() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)], None).unwrap();
        let id = get_articles_for_feed(&conn, feed, 10, 0, true).unwrap()[0].id;
        add_tag_to_article(&conn, id, "Keep", "#4899ec").unwrap();

        delete_feed(&conn, feed).unwrap();

        // Same rule as retention: a tag means keep, and search still finds it.
        assert!(get_saved_articles(&conn, 10, 0, true).unwrap().is_empty());
        assert_eq!(get_tags_for_article(&conn, id).unwrap().len(), 1);
        let feed_id: i64 = conn
            .query_row("SELECT feed_id FROM articles WHERE id = ?1", [id], |r| r.get(0))
            .unwrap();
        assert_eq!(feed_id, SAVED_FEED_ID);
    }

    #[test]
    fn latest_hides_articles_kept_from_a_deleted_feed() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)], None).unwrap();

        let id = get_articles_for_feed(&conn, feed, 10, 0, true).unwrap()[0].id;
        update_article_saved(&conn, id, true).unwrap();
        delete_feed(&conn, feed).unwrap();

        // The row survives for Read Later, but it has no feed to attribute it
        // to, so the Latest view must not show it.
        assert!(get_latest_articles(&conn, 0, 10, 0, true).unwrap().is_empty());
        assert_eq!(get_saved_articles(&conn, 10, 0, true).unwrap().len(), 1);
    }

    #[test]
    fn search_matches_all_terms_in_any_order() {
        let mut conn = test_conn();
        let feed = create_feed(&conn, "Feed", "https://example.com/rss", None, "rss").unwrap();
        batch_insert_articles(&mut conn, &[sample_article(feed)], None).unwrap();

        // "Summary" is in the summary column and "Title" in the title column;
        // a phrase search would never match either order.
        assert_eq!(search_articles(&conn, "Summary Title", 10, 0, true).unwrap().len(), 1);
        assert!(search_articles(&conn, "   ", 10, 0, true).unwrap().is_empty());
    }
}
