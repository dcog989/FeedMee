import type { Article, Feed, Folder } from "../types";

/** Shared test data, so a new field on these types is added once. */
export function makeFeed(id: number, overrides: Partial<Feed> = {}): Feed {
  return {
    id,
    name: `Feed ${id}`,
    folder_id: 1,
    unread_count: 0,
    has_error: false,
    error_count: 0,
    source_type: "rss",
    display_url: `https://example.com/${id}`,
    source_id: String(id),
    ...overrides,
  };
}

export function makeFolder(id: number, feeds: Feed[] = []): Folder {
  return { id, name: `Folder ${id}`, feeds };
}

export function makeArticle(id: number, overrides: Partial<Article> = {}): Article {
  return {
    id,
    feed_id: 1,
    title: `Article ${id}`,
    summary: "",
    author: "",
    url: `https://example.com/${id}`,
    image_url: "",
    timestamp: 0,
    is_read: false,
    is_saved: false,
    has_tags: false,
    ...overrides,
  };
}
