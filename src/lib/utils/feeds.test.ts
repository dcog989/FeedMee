import { describe, expect, it } from "vitest";
import type { Feed, Folder } from "../types";
import { resolveVisibleFeedIds, selectStaleFeeds } from "./feeds";

function feed(id: number, overrides: Partial<Feed> = {}): Feed {
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

function folder(id: number, feeds: Feed[]): Folder {
  return { id, name: `Folder ${id}`, feeds };
}

describe("selectStaleFeeds", () => {
  const folders = [folder(1, [feed(1), feed(2), feed(3, { error_count: 10 })])];

  it("returns feeds that are not fresh or updating", () => {
    const result = selectStaleFeeds(folders, {
      isFeedFresh: (id) => id === 1,
      isFeedUpdating: (id) => id === 2,
      failureLimit: 10,
    });
    expect(result.map((f) => f.id)).toEqual([]);
  });

  it("excludes feeds at or above the failure limit", () => {
    const result = selectStaleFeeds(folders, {
      isFeedFresh: () => false,
      isFeedUpdating: () => false,
      failureLimit: 10,
    });
    expect(result.map((f) => f.id)).toEqual([1, 2]);
  });
});

describe("resolveVisibleFeedIds", () => {
  const folders = [folder(1, [feed(10), feed(11)]), folder(2, [feed(20)])];

  it("returns the feeds of the selected folder", () => {
    expect([...resolveVisibleFeedIds(folders, { selectedFolderId: 2, selectedFeedId: null })]).toEqual([20]);
  });

  it("returns the selected feed when no folder is selected", () => {
    expect([...resolveVisibleFeedIds(folders, { selectedFolderId: null, selectedFeedId: 11 })]).toEqual([11]);
  });

  it("returns an empty set when nothing is selected", () => {
    expect(resolveVisibleFeedIds(folders, { selectedFolderId: null, selectedFeedId: null }).size).toBe(0);
  });

  it("returns an empty set when the selected folder does not exist", () => {
    expect(resolveVisibleFeedIds(folders, { selectedFolderId: 99, selectedFeedId: null }).size).toBe(0);
  });
});
