use opml::{Body, Head, OPML, Outline};

use crate::models::{Feed, Folder};

const OPML_VERSION: &str = "2.0";
const EXPORT_TITLE: &str = "FeedMee Export";

pub fn render(folders: &[Folder]) -> Result<String, String> {
    let document = OPML {
        version: OPML_VERSION.to_string(),
        head: Some(Head {
            title: Some(EXPORT_TITLE.to_string()),
            ..Head::default()
        }),
        body: Body {
            outlines: folders.iter().flat_map(outlines_for_folder).collect(),
        },
    };
    let xml = document
        .to_string()
        .map_err(|e| format!("Failed to write OPML: {}", e))?;
    Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", xml))
}

/// Folder 0 is the synthetic bucket for unfiled feeds; it is flattened into the
/// top level instead of being written as a folder.
fn outlines_for_folder(folder: &Folder) -> Vec<Outline> {
    if folder.id == 0 {
        return folder.feeds.iter().map(feed_outline).collect();
    }
    vec![Outline {
        text: folder.name.clone(),
        outlines: folder.feeds.iter().map(feed_outline).collect(),
        ..Outline::default()
    }]
}

fn feed_outline(feed: &Feed) -> Outline {
    Outline {
        text: feed.name.clone(),
        r#type: Some(feed.feed_type.clone()),
        xml_url: Some(feed.url.clone()),
        ..Outline::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{Feed, Folder};

    use super::render;

    fn feed(name: &str, url: &str, feed_type: &str) -> Feed {
        Feed {
            id: 1,
            name: name.to_string(),
            url: url.to_string(),
            folder_id: None,
            unread_count: 0,
            has_error: false,
            error_count: 0,
            feed_type: feed_type.to_string(),
            display_url: url.to_string(),
            source_id: url.to_string(),
        }
    }

    fn folder(id: i64, name: &str, feeds: Vec<Feed>) -> Folder {
        Folder {
            id,
            name: name.to_string(),
            feeds,
        }
    }

    #[test]
    fn keeps_types_escaping_and_empty_folders() {
        let folders = vec![
            folder(1, "News", vec![feed("F&F", "https://example.com/f", "rss")]),
            folder(0, "", vec![feed("Root", "https://example.com/r", "website")]),
            folder(2, "Empty", vec![]),
        ];

        let xml = render(&folders).unwrap();
        let parsed = opml::OPML::from_str(&xml).unwrap();

        let news = parsed
            .body
            .outlines
            .iter()
            .find(|o| o.text == "News")
            .expect("folder outline");
        assert_eq!(news.outlines[0].text, "F&F");
        assert_eq!(news.outlines[0].xml_url.as_deref(), Some("https://example.com/f"));
        assert_eq!(news.outlines[0].r#type.as_deref(), Some("rss"));

        let root = parsed
            .body
            .outlines
            .iter()
            .find(|o| o.text == "Root")
            .expect("unfiled feed");
        assert_eq!(root.r#type.as_deref(), Some("website"));

        let empty = parsed
            .body
            .outlines
            .iter()
            .find(|o| o.text == "Empty")
            .expect("empty folder");
        assert!(empty.outlines.is_empty());
    }
}
