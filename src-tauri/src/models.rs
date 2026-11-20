use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct Article {
    pub id: i64,
    pub feed_id: i64,
    pub title: String,
    pub summary: String,
    pub author: String,
    pub url: String,
    pub image_url: String,
    /// True when `image_url` comes from a feed-declared low-resolution source
    /// (e.g. a small `media:content` thumbnail) and should be upgraded via
    /// og:image scraping. Not persisted or sent to the frontend.
    #[serde(skip)]
    pub image_low_res: bool,
    pub timestamp: i64,
    pub is_read: bool,
    pub is_saved: bool,
    pub has_tags: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct Feed {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub folder_id: Option<i64>,
    pub unread_count: i64,
    pub has_error: bool,
    pub error_count: i64,
    #[serde(rename = "source_type")]
    pub feed_type: String,
    pub display_url: String,
    pub source_id: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct Folder {
    pub id: i64,
    pub name: String,
    pub feeds: Vec<Feed>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::{Article, Feed};

    fn article() -> Article {
        Article {
            id: 1,
            feed_id: 2,
            title: "Title".to_string(),
            summary: "Summary".to_string(),
            author: "Author".to_string(),
            url: "https://example.com/a".to_string(),
            image_url: "https://example.com/a.png".to_string(),
            image_low_res: true,
            timestamp: 1000,
            is_read: false,
            is_saved: false,
            has_tags: false,
        }
    }

    fn feed() -> Feed {
        Feed {
            id: 1,
            name: "Feed".to_string(),
            url: "https://example.com/rss".to_string(),
            folder_id: None,
            unread_count: 0,
            has_error: false,
            error_count: 0,
            feed_type: "bluesky".to_string(),
            display_url: "bsky:did:plc:abc".to_string(),
            source_id: "did:plc:abc".to_string(),
        }
    }

    #[test]
    fn feed_uses_the_frontend_name_for_source_type() {
        let json = serde_json::to_value(feed()).unwrap();
        assert_eq!(json["source_type"], "bluesky");
        assert!(json.get("feed_type").is_none());
    }

    #[test]
    fn article_does_not_send_the_scrape_hint() {
        let json = serde_json::to_value(article()).unwrap();
        assert_eq!(json["image_url"], "https://example.com/a.png");
        assert!(json.get("image_low_res").is_none());
    }
}
