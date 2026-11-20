import { withAdded, withRemoved } from "./sets";

export function expandFolder(folderId: number): Set<number> {
  return new Set([folderId]);
}

/** Additive expand, used while dragging so hovering a collapsed folder does not hide the source. */
export function expandFolderKeepingOthers(current: Set<number>, folderId: number): Set<number> {
  return withAdded(current, folderId);
}

export function toggleFolder(current: Set<number>, folderId: number): Set<number> {
  if (!current.has(folderId)) return expandFolder(folderId);
  return withRemoved(current, folderId);
}

export function expandAllFolders(folders: readonly { id: number }[]): Set<number> {
  return new Set(folders.map((f) => f.id));
}

export function collapseAllFolders(): Set<number> {
  return new Set();
}
