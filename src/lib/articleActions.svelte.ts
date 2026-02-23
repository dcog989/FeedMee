import { invoke } from '@tauri-apps/api/core';
import type { AppState } from './storeTypes';
import type { Article } from './types';

export function createArticleActions(state: AppState) {
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

        if (state.selectedFeedId === -1) {
            const cutoff = Math.floor(Date.now() / 1000) - state.latestHours * 3600;
            return await invoke('get_latest_articles', {
                cutoffTimestamp: cutoff,
                limit: state.pageSize,
                offset,
                sortDesc,
            });
        } else if (state.selectedFeedId === -2) {
            return await invoke('get_saved_articles', { limit: state.pageSize, offset, sortDesc });
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

    async function reloadCurrentArticleList() {
        state.articles = [];
        state.page = 0;
        const result = await fetchPage(0);
        state.articles = result || [];
        state.hasMore = (result?.length || 0) === state.pageSize;
    }

    async function loadMore() {
        if (
            (!state.selectedFeedId && !state.selectedFolderId) ||
            !state.hasMore ||
            state.isLoadingArticles
        )
            return;
        state.isLoadingArticles = true;
        const nextPage = state.page + 1;
        try {
            const result = await fetchPage(nextPage);
            if (result && result.length > 0) {
                state.articles = [...state.articles, ...result];
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
            for (const folder of state.folders) {
                const feed = folder.feeds.find((f) => f.id === article.feed_id);
                if (feed && feed.unread_count > 0) {
                    feed.unread_count--;
                    break;
                }
            }
        }
    }

    async function toggleSaved(article: Article) {
        const newState = !article.is_saved;
        article.is_saved = newState;

        if (newState) {
            article.is_read = false;
            invoke('mark_article_read', { id: article.id, read: false }).catch(() => {});
        }

        try {
            await invoke('mark_article_saved', { id: article.id, isSaved: newState });
            await state.refreshFolders();
        } catch {
            article.is_saved = !newState;
        }
    }

    async function fetchFullContent(article: Article): Promise<string | null> {
        try {
            return await invoke<string>('get_article_content', { url: article.url });
        } catch {
            return null;
        }
    }

    return { reloadCurrentArticleList, loadMore, selectArticle, toggleSaved, fetchFullContent };
}
