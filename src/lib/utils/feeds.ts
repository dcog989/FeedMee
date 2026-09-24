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
