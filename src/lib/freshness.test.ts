import { describe, expect, it } from "vitest";
import { createFreshnessHelpers } from "./freshness.svelte";
import type { Feed, Folder } from "./types";

const NOW = Date.now();

function feed(id: number): Feed {
  return {
    id,
    name: `Feed ${id}`,
    folder_id: 1,
    unread_count: 0,
    has_error: false,
    error_count: 0,
    source_type: "rss",
    display_url: "",
    source_id: String(id),
  };
}

function folder(id: number, feeds: Feed[]): Folder {
  return { id, name: `Folder ${id}`, feeds };
}

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
    const helpers = helpersFor([folder(1, [feed(1), feed(2)])], [1]);

    expect(helpers.isFeedFresh(1)).toBe(true);
    expect(helpers.isFeedFresh(2)).toBe(false);
  });

  it("a folder is fresh only when all of its feeds are", () => {
    const folders = [folder(1, [feed(1), feed(2)])];

    expect(helpersFor(folders, [1, 2]).isFolderFresh(1)).toBe(true);
    expect(helpersFor(folders, [1]).isFolderFresh(1)).toBe(false);
  });

  it("an empty folder or empty library is not fresh", () => {
    const helpers = helpersFor([folder(1, [])]);

    expect(helpers.isFolderFresh(1)).toBe(false);
    expect(helpers.isAllFresh()).toBe(false);
  });

  it("the library is fresh when every feed is", () => {
    const folders = [folder(1, [feed(1)]), folder(2, [feed(2)])];

    expect(helpersFor(folders, [1, 2]).isAllFresh()).toBe(true);
    expect(helpersFor(folders, [1]).isAllFresh()).toBe(false);
  });
});
