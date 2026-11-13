import { beforeEach, describe, expect, it, vi } from "vitest";
import { BLOCKED_SCAN_LIMIT, createArticleActions } from "./articleActions.svelte";
import type { ArticleStore } from "./storeTypes";
import type { Article } from "./types";
import { makeArticle } from "./utils/testFixtures";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke }));

vi.mock("./store.svelte", () => ({
  FEED_ID_LATEST: -1,
  FEED_ID_SAVED: -2,
  FEED_ID_TODAY: -3,
}));

function blockedPage(): Article[] {
  return Array.from({ length: 50 }, (_, i) => makeArticle(1000 + i, { title: "spam offer" }));
}

function visiblePage(): Article[] {
  return [makeArticle(1, { title: "one" }), makeArticle(2, { title: "two" })];
}

function makeState(): ArticleStore {
  return {
    articles: [],
    page: 0,
    hasMore: true,
    pageSize: 50,
    sortOrder: "desc",
    searchQuery: "",
    selectedFeedId: 1,
    selectedFolderId: null,
    selectedArticle: null,
    blockedPhrases: ["spam"],
  } as unknown as ArticleStore;
}

describe("reloadCurrentArticleList", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("skips a page that the blocked phrases filtered out", async () => {
    invoke.mockResolvedValueOnce(blockedPage()).mockResolvedValue(visiblePage());
    const state = makeState();

    await createArticleActions(state).reloadCurrentArticleList();

    expect(invoke).toHaveBeenCalledTimes(2);
    expect(state.articles).toHaveLength(2);
    expect(state.page).toBe(1);
    expect(state.hasMore).toBe(false);
  });

  it("stops scanning after the limit when every page is filtered out", async () => {
    invoke.mockResolvedValue(blockedPage());
    const state = makeState();

    await createArticleActions(state).reloadCurrentArticleList();

    expect(invoke).toHaveBeenCalledTimes(BLOCKED_SCAN_LIMIT);
    expect(state.articles).toHaveLength(0);
    expect(state.hasMore).toBe(true);
  });

  it("stops a scan that a newer reload replaced", async () => {
    let release = (_rows: Article[]) => {};
    invoke.mockImplementationOnce(
      () =>
        new Promise<Article[]>((resolve) => {
          release = resolve;
        }),
    );
    invoke.mockResolvedValue(visiblePage());
    const state = makeState();
    const actions = createArticleActions(state);

    const superseded = actions.reloadCurrentArticleList();
    await actions.reloadCurrentArticleList();
    release(blockedPage());
    await superseded;

    expect(invoke).toHaveBeenCalledTimes(2);
    expect(state.articles.map((a) => a.id)).toEqual([1, 2]);
  });
});
