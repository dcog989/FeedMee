import { LS_EXPANDED_FOLDERS } from "./utils/persistence";

export function useExpandedFolders(state: {
  folders: { id: number }[];
  expandedFolders: Set<number>;
  autoCollapseFolders: boolean;
}) {
  // Initialization lives in the store (single owner); this effect only
  // persists changes back to storage.
  $effect(() => {
    localStorage.setItem(LS_EXPANDED_FOLDERS, JSON.stringify(Array.from(state.expandedFolders)));
  });

  function toggleFolder(id: number) {
    const newSet = new Set(state.expandedFolders);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      if (state.autoCollapseFolders) newSet.clear();
      newSet.add(id);
    }
    state.expandedFolders = newSet;
  }

  function expandAll() {
    const newSet = new Set<number>();
    for (const f of state.folders) newSet.add(f.id);
    state.expandedFolders = newSet;
  }

  function collapseAll() {
    state.expandedFolders = new Set();
  }

  return { toggleFolder, expandAll, collapseAll };
}
