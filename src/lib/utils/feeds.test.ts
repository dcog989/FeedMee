import { describe, expect, it } from "vitest";
import { findFeed, findFeedFolderId, resolveVisibleFeedIds, selectStaleFeeds } from "./feeds";
import { makeFeed, makeFolder } from "./testFixtures";

describe("selectStaleFeeds", () => {
  const folders = [makeFolder(1, [makeFeed(1), makeFeed(2), makeFeed(3, { error_count: 10 })])];

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
  const folders = [makeFolder(1, [makeFeed(10), makeFeed(11)]), makeFolder(2, [makeFeed(20)])];

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

describe("findFeedFolderId", () => {
  it("returns 0 for a feed in the uncategorized root folder", () => {
    const folders = [makeFolder(1, [makeFeed(1)]), makeFolder(0, [makeFeed(9, { folder_id: 0 })])];

    expect(findFeedFolderId(folders, 9)).toBe(0);
    expect(findFeedFolderId(folders, 1)).toBe(1);
    expect(findFeedFolderId(folders, 99)).toBeNull();
  });
});

describe("findFeed", () => {
  it("finds a feed by id and returns undefined otherwise", () => {
    const folders = [makeFolder(1, [makeFeed(1)])];

    expect(findFeed(folders, 1)?.id).toBe(1);
    expect(findFeed(folders, 2)).toBeUndefined();
  });
});
