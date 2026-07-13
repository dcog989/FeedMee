import { LS_EXPANDED_FOLDERS } from './utils/persistence';

export function useExpandedFolders(state: {
  folders: { id: number }[];
  expandedFolders: Set<number>;
  autoCollapseFolders: boolean;
}) {
  let initialized = $state(false);

  $effect(() => {
    if (!initialized) {
      const stored = localStorage.getItem(LS_EXPANDED_FOLDERS);
      if (stored) {
        try {
          state.expandedFolders = new Set(JSON.parse(stored));
        } catch (e) {
          console.error(e);
        }
      } else {
        const newSet = new Set<number>();
        for (const f of state.folders) newSet.add(f.id);
        state.expandedFolders = newSet;
      }
      initialized = true;
    }
  });

  $effect(() => {
    if (initialized) {
      localStorage.setItem(LS_EXPANDED_FOLDERS, JSON.stringify(Array.from(state.expandedFolders)));
    }
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
