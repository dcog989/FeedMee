import { invoke } from '@tauri-apps/api/core';
import type { RefreshStore } from './storeTypes';

const REFRESH_CONCURRENCY = 5;

async function runWithConcurrency<T>(items: T[], fn: (item: T) => Promise<void>, concurrency: number) {
  let index = 0;
  const worker = async () => {
    while (index < items.length) {
      const item = items[index++];
      await fn(item);
    }
  };
  await Promise.all(Array.from({ length: concurrency }, () => worker()));
}

export function createFeedRefresher(state: RefreshStore) {
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  function saveLastRefreshed() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      state.persistLastRefreshed();
      saveTimer = null;
    }, 50);
  }

  async function performSingleFeedRefresh(feedId: number) {
    try {
      const unreadCount = await invoke<number>('refresh_feed', { feedId });
      state.lastRefreshed.set(feedId, Date.now());
      saveLastRefreshed();
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
      .filter((f) => !state.isFeedFresh(f.id) && !state.updatingFeedIds.has(f.id));
    if (staleFeeds.length === 0) return;

    state.isRefreshingFeeds = true;

    const addSet = new Set(state.updatingFeedIds);
    for (const f of staleFeeds) addSet.add(f.id);
    state.updatingFeedIds = addSet;

    try {
      await runWithConcurrency(staleFeeds, (feed) => performSingleFeedRefresh(feed.id), REFRESH_CONCURRENCY);
      await state.refreshFolders();
      if (state.selectedFeedId !== null || state.selectedFolderId !== null) {
        await state.reloadCurrentArticleList({ selectTop: true });
      }
    } catch (e) {
      console.error('Failed to refresh all feeds:', e);
    } finally {
      state.isRefreshingFeeds = false;
    }
  }

  async function requestRefreshFeed(feedId: number) {
    if (state.isFeedFresh(feedId) || state.updatingFeedIds.has(feedId)) return;

    // Mark as updating before kicking off the refresh.
    // performSingleFeedRefresh's finally block is the single place that removes it.
    const addSet = new Set(state.updatingFeedIds);
    addSet.add(feedId);
    state.updatingFeedIds = addSet;

    await performSingleFeedRefresh(feedId);

    if (state.selectedFeedId === feedId) {
      await state.reloadCurrentArticleList({ selectTop: true });
    }
  }

  async function requestRefreshFolder(folderId: number) {
    const folder = state.folders.find((f) => f.id === folderId);
    if (!folder || folder.feeds.length === 0) return;

    const staleFeeds = folder.feeds.filter((f) => !state.isFeedFresh(f.id) && !state.updatingFeedIds.has(f.id));
    if (staleFeeds.length === 0) return;

    const addSet = new Set(state.updatingFeedIds);
    for (const f of staleFeeds) addSet.add(f.id);
    state.updatingFeedIds = addSet;

    try {
      await runWithConcurrency(staleFeeds, (feed) => performSingleFeedRefresh(feed.id), REFRESH_CONCURRENCY);
      await state.refreshFolders();
      if (state.selectedFolderId === folderId || folder.feeds.some((f) => f.id === state.selectedFeedId)) {
        await state.reloadCurrentArticleList({ selectTop: true });
      }
    } catch (e) {
      console.error(`Failed to refresh folder ${folderId}:`, e);
    }
  }

  return { refreshAllFeeds, requestRefreshFeed, requestRefreshFolder };
}
