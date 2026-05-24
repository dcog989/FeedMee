import { invoke } from '@tauri-apps/api/core';
import type { AppState } from './storeTypes';

const REFRESH_CONCURRENCY = 5;

export function createFeedRefresher(state: AppState) {
    function saveLastRefreshed() {
        const obj = Object.fromEntries(state.lastRefreshed);
        localStorage.setItem('lastRefreshed', JSON.stringify(obj));
    }

    async function performSingleFeedRefresh(feedId: number) {
        try {
            await invoke('refresh_feed', { feedId });
            state.lastRefreshed = new Map(state.lastRefreshed).set(feedId, Date.now());
            saveLastRefreshed();
            const unreadCount = await invoke<number>('get_feed_unread_count', { feedId });
            for (const folder of state.folders) {
                const feed = folder.feeds.find((f) => f.id === feedId);
                if (feed) {
                    feed.unread_count = unreadCount;
                    break;
                }
            }
        } catch (e) {
            console.error(`Failed to refresh feed ${feedId}:`, e);
        } finally {
            // Remove from updating set — this is the single place that does it
            const newSet = new Set(state.updatingFeedIds);
            newSet.delete(feedId);
            state.updatingFeedIds = newSet;
        }
    }

    async function refreshAllFeeds() {
        const staleFeeds = state.folders
            .flatMap((f) => f.feeds)
            .filter((f) => !state.isFeedFresh(f.id));
        if (staleFeeds.length === 0) return;

        state.isRefreshingFeeds = true;

        const addSet = new Set(state.updatingFeedIds);
        for (const f of staleFeeds) addSet.add(f.id);
        state.updatingFeedIds = addSet;

        let index = 0;
        const worker = async () => {
            while (index < staleFeeds.length) {
                const feed = staleFeeds[index++];
                await performSingleFeedRefresh(feed.id);
            }
        };

        const workers = Array(REFRESH_CONCURRENCY)
            .fill(null)
            .map(() => worker());

        try {
            await Promise.all(workers);
            await state.refreshFolders();
            if (state.selectedFeedId !== null || state.selectedFolderId !== null) {
                await state.reloadCurrentArticleList();
            }
        } catch (e) {
            console.error('Failed to refresh all feeds:', e);
        } finally {
            state.updatingFeedIds = new Set();
            state.isRefreshingFeeds = false;
        }
    }

    async function requestRefreshFeed(feedId: number) {
        if (state.isFeedFresh(feedId)) return;

        // Mark as updating before kicking off the refresh.
        // performSingleFeedRefresh's finally block is the single place that removes it.
        const addSet = new Set(state.updatingFeedIds);
        addSet.add(feedId);
        state.updatingFeedIds = addSet;

        await performSingleFeedRefresh(feedId);

        if (state.selectedFeedId === feedId) {
            await state.reloadCurrentArticleList();
        }
    }

    async function requestRefreshFolder(folderId: number) {
        const folder = state.folders.find((f) => f.id === folderId);
        if (!folder || folder.feeds.length === 0) return;

        const staleFeeds = folder.feeds.filter((f) => !state.isFeedFresh(f.id));
        if (staleFeeds.length === 0) return;

        const addSet = new Set(state.updatingFeedIds);
        for (const f of staleFeeds) addSet.add(f.id);
        state.updatingFeedIds = addSet;

        let index = 0;
        const worker = async () => {
            while (index < staleFeeds.length) {
                const feed = staleFeeds[index++];
                await performSingleFeedRefresh(feed.id);
            }
        };

        const workers = Array(REFRESH_CONCURRENCY)
            .fill(null)
            .map(() => worker());

        try {
            await Promise.all(workers);
            await state.refreshFolders();
            if (
                state.selectedFolderId === folderId ||
                folder.feeds.some((f) => f.id === state.selectedFeedId)
            ) {
                await state.reloadCurrentArticleList();
            }
        } catch (e) {
            console.error(`Failed to refresh folder ${folderId}:`, e);
        }
    }

    return { refreshAllFeeds, requestRefreshFeed, requestRefreshFolder };
}
