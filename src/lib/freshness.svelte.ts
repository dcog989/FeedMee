import type { Folder } from "./types";
import { LS_LAST_REFRESHED } from "./utils/persistence";

interface FreshnessDeps {
  folders: Folder[];
  updatingFeedIds: Set<number>;
  lastRefreshed: Map<number, number>;
  readonly debounceMs: number;
}

export function createFreshnessHelpers(state: FreshnessDeps) {
  function adjustUnreadCount(feedId: number, delta: number) {
    for (const folder of state.folders) {
      const feed = folder.feeds.find((f) => f.id === feedId);
      if (feed) {
        feed.unread_count = Math.max(0, feed.unread_count + delta);
        break;
      }
    }
  }

  function isFeedUpdating(feedId: number) {
    return state.updatingFeedIds.has(feedId);
  }

  function isFolderUpdating(folderId: number) {
    const folder = state.folders.find((f) => f.id === folderId);
    if (!folder) return false;
    return folder.feeds.some((feed) => state.updatingFeedIds.has(feed.id));
  }

  function persistLastRefreshed() {
    const obj = Object.fromEntries(state.lastRefreshed);
    localStorage.setItem(LS_LAST_REFRESHED, JSON.stringify(obj));
  }

  function isFeedFresh(feedId: number): boolean {
    return Date.now() - (state.lastRefreshed.get(feedId) || 0) < state.debounceMs;
  }

  function isFolderFresh(folderId: number): boolean {
    const folder = state.folders.find((f) => f.id === folderId);
    if (!folder || folder.feeds.length === 0) return false;
    return folder.feeds.every((f) => isFeedFresh(f.id));
  }

  function isAllFresh(): boolean {
    return state.folders.flatMap((f) => f.feeds).every((f) => isFeedFresh(f.id));
  }

  return {
    adjustUnreadCount,
    isFeedUpdating,
    isFolderUpdating,
    persistLastRefreshed,
    isFeedFresh,
    isFolderFresh,
    isAllFresh,
  };
}
