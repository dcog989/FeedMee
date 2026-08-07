import { invoke } from '@tauri-apps/api/core';
import { FEED_ID_LATEST, FEED_ID_SAVED, FEED_ID_TODAY } from './store.svelte';
import type { ArticleStore } from './storeTypes';
import type { Article } from './types';

function matchesBlockedPhrases(article: Article, phrases: string[]): boolean {
  if (phrases.length === 0) return false;
  const text = `${article.title} ${article.summary} ${article.author}`.toLowerCase();
  return phrases.some((phrase) => phrase && text.includes(phrase.toLowerCase()));
}

export function createArticleActions(state: ArticleStore) {
  async function fetchPage(page: number): Promise<Article[]> {
    const offset = page * state.pageSize;
    const sortDesc = state.sortOrder === 'desc';

    if (state.searchQuery.trim()) {
      return await invoke('search_articles', {
        query: state.searchQuery.trim(),
        limit: state.pageSize,
        offset,
        sortDesc,
      });
    }

    if (state.selectedFeedId === FEED_ID_LATEST) {
      const cutoff = Math.floor(Date.now() / 1000) - state.latestHours * 3600;
      return await invoke('get_latest_articles', {
        cutoffTimestamp: cutoff,
        limit: state.pageSize,
        offset,
        sortDesc,
      });
    } else if (state.selectedFeedId === FEED_ID_TODAY) {
      const now = new Date();
      const midnight = new Date(now.getFullYear(), now.getMonth(), now.getDate());
      const cutoff = Math.floor(midnight.getTime() / 1000);
      return await invoke('get_latest_articles', {
        cutoffTimestamp: cutoff,
        limit: state.pageSize,
        offset,
        sortDesc,
      });
    } else if (state.selectedFeedId === FEED_ID_SAVED) {
      return await invoke('get_saved_articles', {
        limit: state.pageSize,
        offset,
        sortDesc,
      });
    } else if (state.selectedFeedId) {
      return await invoke('get_articles_for_feed', {
        feedId: state.selectedFeedId,
        limit: state.pageSize,
        offset,
        sortDesc,
      });
    } else if (state.selectedFolderId) {
      return await invoke('get_articles_for_folder', {
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

  async function reloadCurrentArticleList(options?: { selectTop?: boolean }) {
    const gen = ++reloadGeneration;
    state.articles = [];
    state.page = 0;
    const result = await fetchPage(0);
    if (gen !== reloadGeneration) return;
    state.articles = filterBlocked(result || []);
    state.hasMore = (result?.length || 0) === state.pageSize;

    if (options?.selectTop) {
      if (state.articles.length > 0) {
        state.selectedArticle = state.articles[0];
        state.focusedPane = 'reading';
      }
    } else if (state.selectedArticle) {
      const fresh = state.articles.find((a) => a.id === state.selectedArticle?.id);
      if (fresh) state.selectedArticle = fresh;
    }
  }

  async function loadMore() {
    if ((state.selectedFeedId === null && state.selectedFolderId === null) || !state.hasMore || state.isLoadingArticles)
      return;
    state.isLoadingArticles = true;
    const nextPage = state.page + 1;
    try {
      const result = await fetchPage(nextPage);
      if (result && result.length > 0) {
        const filtered = filterBlocked(result);
        state.articles = [...state.articles, ...filtered];
        state.page = nextPage;
        state.hasMore = result.length === state.pageSize;
      } else {
        state.hasMore = false;
      }
    } catch (e) {
      console.error('Failed to load more articles:', e);
    } finally {
      state.isLoadingArticles = false;
    }
  }

  function selectArticle(article: Article) {
    state.focusedPane = 'list';
    state.selectedArticle = article;
    if (!article.is_read) {
      article.is_read = true;
      invoke('mark_article_read', { id: article.id, read: true }).catch(() => {
        article.is_read = false;
      });
      state.adjustUnreadCount(article.feed_id, -1);
    }
  }

  async function toggleSaved(article: Article) {
    const newSaved = !article.is_saved;
    article.is_saved = newSaved;

    try {
      await invoke('mark_article_saved', { id: article.id, isSaved: newSaved });
      await state.refreshFolders();
    } catch {
      article.is_saved = !newSaved;
    }
  }

  async function fetchFullContent(article: Article): Promise<string | null> {
    try {
      return await invoke<string>('get_article_content', { url: article.url });
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
  };
}
