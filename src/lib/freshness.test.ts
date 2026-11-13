import { describe, expect, it } from "vitest";
import { createFreshnessHelpers } from "./freshness.svelte";
import type { Folder } from "./types";
import { makeFeed, makeFolder } from "./utils/testFixtures";

const NOW = Date.now();

function helpersFor(folders: Folder[], refreshed: number[] = []) {
  return createFreshnessHelpers({
    folders,
    updatingFeedIds: new Set<number>(),
    lastRefreshed: new Map(refreshed.map((id): [number, number] => [id, NOW])),
    debounceMs: 60_000,
  });
}

describe("freshness", () => {
  it("treats a feed refreshed inside the window as fresh", () => {
    const helpers = helpersFor([makeFolder(1, [makeFeed(1), makeFeed(2)])], [1]);

    expect(helpers.isFeedFresh(1)).toBe(true);
    expect(helpers.isFeedFresh(2)).toBe(false);
  });

  it("a folder is fresh only when all of its feeds are", () => {
    const folders = [makeFolder(1, [makeFeed(1), makeFeed(2)])];

    expect(helpersFor(folders, [1, 2]).isFolderFresh(1)).toBe(true);
    expect(helpersFor(folders, [1]).isFolderFresh(1)).toBe(false);
  });

  it("an empty folder is fresh, while an empty library is not", () => {
    const helpers = helpersFor([makeFolder(1)]);

    expect(helpers.isFolderFresh(1)).toBe(true);
    expect(helpers.isAllFresh()).toBe(false);
  });

  it("the library is fresh when every feed is", () => {
    const folders = [makeFolder(1, [makeFeed(1)]), makeFolder(2, [makeFeed(2)])];

    expect(helpersFor(folders, [1, 2]).isAllFresh()).toBe(true);
    expect(helpersFor(folders, [1]).isAllFresh()).toBe(false);
  });
});
