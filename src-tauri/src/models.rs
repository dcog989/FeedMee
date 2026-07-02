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
    pub feed_type: String,
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
    pub color: String,
}
