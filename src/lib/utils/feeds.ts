import type { Feed, Folder } from "../types";

export function findFeed(folders: Folder[], feedId: number): Feed | undefined {
  for (const folder of folders) {
    const feed = folder.feeds.find((f) => f.id === feedId);
    if (feed) return feed;
  }
  return undefined;
}

export function findFeedFolderId(folders: Folder[], feedId: number): number | null {
  for (const folder of folders) {
    if (folder.feeds.some((f) => f.id === feedId)) return folder.id;
  }
  return null;
}

export interface FeedRefreshFilters {
  isFeedFresh(feedId: number): boolean;
  isFeedUpdating(feedId: number): boolean;
  failureLimit: number;
}

export function selectStaleFeeds(folders: Folder[], filters: FeedRefreshFilters): Feed[] {
  return folders
    .flatMap((f) => f.feeds)
    .filter((f) => !filters.isFeedFresh(f.id) && !filters.isFeedUpdating(f.id) && f.error_count < filters.failureLimit);
}

export function resolveVisibleFeedIds(
  folders: Folder[],
  selection: { selectedFolderId: number | null; selectedFeedId: number | null },
): Set<number> {
  const ids = new Set<number>();
  if (selection.selectedFolderId !== null) {
    const folder = folders.find((f) => f.id === selection.selectedFolderId);
    if (folder) for (const feed of folder.feeds) ids.add(feed.id);
  } else if (selection.selectedFeedId !== null) {
    ids.add(selection.selectedFeedId);
  }
  return ids;
}
