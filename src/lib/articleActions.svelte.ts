import { invoke } from "@tauri-apps/api/core";
import { FEED_ID_LATEST, FEED_ID_SAVED, FEED_ID_TODAY } from "./store.svelte";
import type { ArticleStore } from "./storeTypes";
import type { Article } from "./types";
import { mergeArticles } from "./utils/articles";

/** Pages to scan for one visible page; blocked phrases can hide whole pages. */
export const BLOCKED_SCAN_LIMIT = 10;

function matchesBlockedPhrases(article: Article, phrases: string[]): boolean {
  if (phrases.length === 0) return false;
  const text = `${article.title} ${article.summary} ${article.author}`.toLowerCase();
  return phrases.some((phrase) => phrase && text.includes(phrase.toLowerCase()));
}

export function createArticleActions(state: ArticleStore) {
  async function fetchPage(page: number): Promise<Article[]> {
    const offset = page * state.pageSize;
    const sortDesc = state.sortOrder === "desc";

    if (state.searchQuery.trim()) {
      return await invoke("search_articles", {
        query: state.searchQuery.trim(),
        limit: state.pageSize,
        offset,
        sortDesc,
      });
    }

    if (state.selectedFeedId === FEED_ID_LATEST) {
      const cutoff = Math.floor(Date.now() / 1000) - state.latestHours * 3600;
      return await invoke("get_latest_articles", {
        cutoffTimestamp: cutoff,
        limit: state.pageSize,
        offset,
        sortDesc,
      });
    } else if (state.selectedFeedId === FEED_ID_TODAY) {
      const now = new Date();
      const midnight = new Date(now.getFullYear(), now.getMonth(), now.getDate());
      const cutoff = Math.floor(midnight.getTime() / 1000);
      return await invoke("get_latest_articles", {
        cutoffTimestamp: cutoff,
        limit: state.pageSize,
        offset,
        sortDesc,
      });
    } else if (state.selectedFeedId === FEED_ID_SAVED) {
      return await invoke("get_saved_articles", {
        limit: state.pageSize,
        offset,
        sortDesc,
      });
    } else if (state.selectedFeedId) {
      return await invoke("get_articles_for_feed", {
        feedId: state.selectedFeedId,
        limit: state.pageSize,
        offset,
        sortDesc,
      });
    } else if (state.selectedFolderId) {
      return await invoke("get_articles_for_folder", {
        folderId: state.selectedFolderId,
        limit: state.pageSize,
        offset,
        sortDesc,
      });
    }
    return [];
  }

  function filterBlocked(articles: Article[]): Article[] {
    if (state.blockedPhrases.length === 0) return articles;
    return articles.filter((a) => !matchesBlockedPhrases(a, state.blockedPhrases));
  }

  let reloadGeneration = 0;

  // Fetches pages until one has articles left after blocked-phrase filtering.
  // Without this, a fully filtered page leaves an empty list, so the scroll
  // handler that owns pagination never fires again. Scans at most
  // BLOCKED_SCAN_LIMIT pages and reports hasMore so scrolling can continue.
  async function fetchVisiblePage(
    fromPage: number,
    gen: number,
  ): Promise<{ articles: Article[]; page: number; hasMore: boolean }> {
    let page = fromPage;
    for (let scanned = 0; scanned < BLOCKED_SCAN_LIMIT; scanned += 1) {
      const result = (await fetchPage(page)) ?? [];
      // A newer reload superseded this scan; stop querying.
      if (gen !== reloadGeneration) return { articles: [], page, hasMore: false };
      const hasMore = result.length === state.pageSize;
      const articles = filterBlocked(result);
      if (articles.length > 0 || !hasMore) return { articles, page, hasMore };
      page += 1;
    }
    return { articles: [], page, hasMore: true };
  }

  async function reloadCurrentArticleList() {
    const gen = ++reloadGeneration;
    state.isLoadingArticles = true;
    state.articles = [];
    state.page = 0;
    try {
      const page = await fetchVisiblePage(0, gen);
      if (gen !== reloadGeneration) return;
      state.articles = page.articles;
      state.page = page.page;
      state.hasMore = page.hasMore;

      if (state.selectedArticle) {
        const fresh = state.articles.find((a) => a.id === state.selectedArticle?.id);
        state.selectedArticle = fresh ?? state.articles[0] ?? null;
      }
    } catch (e) {
      if (gen === reloadGeneration) state.hasMore = false;
      console.error("Failed to reload article list:", e);
    } finally {
      // A newer reload owns the flag when this one was superseded.
      if (gen === reloadGeneration) state.isLoadingArticles = false;
    }
  }

  async function loadMore() {
    if ((state.selectedFeedId === null && state.selectedFolderId === null) || !state.hasMore || state.isLoadingArticles)
      return;
    state.isLoadingArticles = true;
    const gen = reloadGeneration;
    try {
      const page = await fetchVisiblePage(state.page + 1, gen);
      if (gen !== reloadGeneration) return;
      state.articles = mergeArticles(state.articles, page.articles);
      state.page = page.page;
      state.hasMore = page.hasMore;
    } catch (e) {
      console.error("Failed to load more articles:", e);
    } finally {
      state.isLoadingArticles = false;
    }
  }

  function setArticleRead(article: Article, read: boolean) {
    if (article.is_read === read) return;
    article.is_read = read;
    state.adjustUnreadCount(article.feed_id, read ? -1 : 1);
    invoke("mark_article_read", { id: article.id, read }).catch((e) => {
      article.is_read = !read;
      state.adjustUnreadCount(article.feed_id, read ? 1 : -1);
      console.error("mark_article_read failed:", e);
    });
  }

  function markRead(article: Article) {
    setArticleRead(article, true);
  }

  function selectArticle(article: Article) {
    state.focusedPane = "list";
    const previous = state.selectedArticle;
    state.selectedArticle = article;
    // Mark the article you leave, not the one you open, so the open one stays unread.
    if (previous && previous.id !== article.id) markRead(previous);
  }

  async function toggleSaved(article: Article) {
    const newSaved = !article.is_saved;
    article.is_saved = newSaved;

    try {
      await invoke("mark_article_saved", { id: article.id, isSaved: newSaved });
    } catch (e) {
      article.is_saved = !newSaved;
      state.alert("Could not update Read Later.");
      console.error("mark_article_saved failed:", e);
    }
  }

  async function fetchFullContent(article: Article): Promise<string | null> {
    try {
      return await invoke<string>("get_article_content", { url: article.url });
    } catch {
      return null;
    }
  }

  return {
    reloadCurrentArticleList,
    loadMore,
    selectArticle,
    toggleSaved,
    fetchFullContent,
    setArticleRead,
  };
}
