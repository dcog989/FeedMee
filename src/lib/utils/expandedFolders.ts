import { withAdded, withRemoved } from "./sets";

export function expandFolder(current: Set<number>, folderId: number, autoCollapse: boolean): Set<number> {
  if (autoCollapse) return new Set([folderId]);
  return withAdded(current, folderId);
}

export function toggleFolder(current: Set<number>, folderId: number, autoCollapse: boolean): Set<number> {
  if (!current.has(folderId)) return expandFolder(current, folderId, autoCollapse);
  return withRemoved(current, folderId);
}

export function expandAllFolders(folders: readonly { id: number }[]): Set<number> {
  return new Set(folders.map((f) => f.id));
}

export function collapseAllFolders(): Set<number> {
  return new Set();
}
