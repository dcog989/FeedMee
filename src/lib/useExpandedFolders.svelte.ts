import { collapseAllFolders, expandAllFolders, toggleFolder as toggle } from "./utils/expandedFolders";
import { LS_EXPANDED_FOLDERS, writeJson } from "./utils/persistence";

export function useExpandedFolders(state: {
  folders: { id: number }[];
  expandedFolders: Set<number>;
  autoCollapseFolders: boolean;
}) {
  // Initialization lives in the store (single owner); this effect only
  // persists changes back to storage.
  $effect(() => {
    writeJson(LS_EXPANDED_FOLDERS, Array.from(state.expandedFolders));
  });

  function toggleFolder(id: number) {
    state.expandedFolders = toggle(state.expandedFolders, id, state.autoCollapseFolders);
  }

  function expandAll() {
    state.expandedFolders = expandAllFolders(state.folders);
  }

  function collapseAll() {
    state.expandedFolders = collapseAllFolders();
  }

  return { toggleFolder, expandAll, collapseAll };
}
