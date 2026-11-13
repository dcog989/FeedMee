import { beforeEach, describe, expect, it, vi } from "vitest";
import { createFeedRefresher } from "./feedRefresh.svelte";
import type { RefreshStore } from "./storeTypes";
import { makeFeed, makeFolder } from "./utils/testFixtures";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const DEBOUNCE_MS = 60_000;

function makeHarness(folders = [makeFolder(1, [makeFeed(1), makeFeed(2)])], freshIds: number[] = []) {
  const lastRefreshed = new Map(freshIds.map((id): [number, number] => [id, Date.now()]));
  const state = {
    folders,
    updatingFeedIds: new Set<number>(),
    lastRefreshed,
    debounceMs: DEBOUNCE_MS,
    isRefreshingFeeds: false,
    selectedFeedId: null,
    selectedFolderId: null,
    persistLastRefreshed: () => {},
    refreshFolders: vi.fn(async () => {}),
    reloadCurrentArticleList: vi.fn(async () => {}),
    isFeedFresh: (feedId: number) => Date.now() - (lastRefreshed.get(feedId) ?? 0) < DEBOUNCE_MS,
  } as unknown as RefreshStore;
  // Reads the current set, which the refresher replaces on every change.
  state.isFeedUpdating = (feedId: number) => state.updatingFeedIds.has(feedId);

  return { state, refresher: createFeedRefresher(state) };
}

beforeEach(() => {
  invoke.mockReset();
});

describe("refreshAllFeeds", () => {
  it("refreshes every stale feed once and clears the updating set", async () => {
    invoke.mockResolvedValue(3);
    const { state, refresher } = makeHarness();

    await refresher.refreshAllFeeds();

    expect(invoke).toHaveBeenCalledTimes(2);
    expect(invoke).toHaveBeenCalledWith("refresh_feed", { feedId: 1 });
    expect(invoke).toHaveBeenCalledWith("refresh_feed", { feedId: 2 });
    expect(state.updatingFeedIds.size).toBe(0);
    expect(state.isRefreshingFeeds).toBe(false);
    expect(state.folders[0].feeds[0].unread_count).toBe(3);
  });

  it("does nothing when every feed is fresh", async () => {
    const { state, refresher } = makeHarness(undefined, [1, 2]);

    await refresher.refreshAllFeeds();

    expect(invoke).not.toHaveBeenCalled();
    expect(state.isRefreshingFeeds).toBe(false);
  });

  it("keeps the updating set clean when one feed fails", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    invoke.mockImplementation((_command, args: { feedId: number }) =>
      args.feedId === 1 ? Promise.reject("boom") : Promise.resolve(4),
    );
    const { state, refresher } = makeHarness();

    await refresher.refreshAllFeeds();

    expect(invoke).toHaveBeenCalledTimes(2);
    expect(state.updatingFeedIds.size).toBe(0);
    expect(state.folders[0].feeds[0].unread_count).toBe(0);
    expect(state.folders[0].feeds[1].unread_count).toBe(4);
    consoleError.mockRestore();
  });
});

describe("requestRefreshFeed", () => {
  it("ignores a second request while the feed is refreshing", async () => {
    let release: (count: number) => void = () => {};
    invoke.mockImplementationOnce(
      () =>
        new Promise<number>((resolve) => {
          release = resolve;
        }),
    );
    const { state, refresher } = makeHarness([makeFolder(1, [makeFeed(1)])]);

    const first = refresher.requestRefreshFeed(1);
    await refresher.requestRefreshFeed(1);

    expect(invoke).toHaveBeenCalledTimes(1);

    release(5);
    await first;

    expect(state.updatingFeedIds.size).toBe(0);
  });
});
