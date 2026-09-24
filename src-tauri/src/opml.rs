use std::fmt::Write;

use crate::models::Folder;

pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn render(folders: &[Folder]) -> String {
    let mut opml = String::new();
    let _ = writeln!(&mut opml, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    let _ = writeln!(&mut opml, "<opml version=\"2.0\">");
    let _ = writeln!(&mut opml, "  <head><title>FeedMee Export</title></head>");
    let _ = writeln!(&mut opml, "  <body>");

    for folder in folders {
        if folder.feeds.is_empty() {
            continue;
        }
        if folder.id == 0 {
            for feed in &folder.feeds {
                let _ = writeln!(
                    &mut opml,
                    "      <outline type=\"rss\" text=\"{}\" xmlUrl=\"{}\" />",
                    escape(&feed.name),
                    escape(&feed.url)
                );
            }
        } else {
            let _ = writeln!(&mut opml, "    <outline text=\"{}\">", escape(&folder.name));
            for feed in &folder.feeds {
                let _ = writeln!(
                    &mut opml,
                    "      <outline type=\"rss\" text=\"{}\" xmlUrl=\"{}\" />",
                    escape(&feed.name),
                    escape(&feed.url)
                );
            }
            let _ = writeln!(&mut opml, "    </outline>");
        }
    }
    let _ = writeln!(&mut opml, "  </body>");
    let _ = writeln!(&mut opml, "</opml>");
    opml
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Feed;

    fn feed(name: &str, url: &str) -> Feed {
        Feed {
            id: 1,
            name: name.to_string(),
            url: url.to_string(),
            folder_id: None,
            unread_count: 0,
            has_error: false,
            error_count: 0,
            feed_type: "rss".to_string(),
            display_url: url.to_string(),
            source_id: url.to_string(),
        }
    }

    #[test]
    fn escapes_xml_special_characters() {
        assert_eq!(
            escape("a & b < c > d \" e ' f"),
            "a &amp; b &lt; c &gt; d &quot; e &apos; f"
        );
    }

    #[test]
    fn renders_folders_and_root_feeds() {
        let folders = vec![
            Folder {
                id: 1,
                name: "News".to_string(),
                feeds: vec![feed("F&F", "https://example.com/f")],
            },
            Folder {
                id: 0,
                name: String::new(),
                feeds: vec![feed("Root", "https://example.com/r")],
            },
        ];

        let out = render(&folders);
        assert!(out.contains("<outline text=\"News\">"));
        assert!(out.contains("text=\"F&amp;F\""));
        assert!(out.contains("xmlUrl=\"https://example.com/f\""));
        assert!(out.contains("text=\"Root\""));
        assert!(out.trim_end().ends_with("</opml>"));
    }
}
