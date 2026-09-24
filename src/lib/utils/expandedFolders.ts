export function expandFolder(current: Set<number>, folderId: number, autoCollapse: boolean): Set<number> {
  if (autoCollapse) return new Set([folderId]);
  const next = new Set(current);
  next.add(folderId);
  return next;
}

export function toggleFolder(current: Set<number>, folderId: number, autoCollapse: boolean): Set<number> {
  if (!current.has(folderId)) return expandFolder(current, folderId, autoCollapse);
  const next = new Set(current);
  next.delete(folderId);
  return next;
}

export function expandAllFolders(folders: readonly { id: number }[]): Set<number> {
  return new Set(folders.map((f) => f.id));
}

export function collapseAllFolders(): Set<number> {
  return new Set();
}
