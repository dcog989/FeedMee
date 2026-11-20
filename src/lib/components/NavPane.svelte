<script lang="ts">
import { Info, Settings } from "@lucide/svelte";
import { navStore, uiStore } from "#lib/store.svelte.js";
import { useExpandedFolders } from "#lib/useExpandedFolders.svelte.js";
import { expandFolderKeepingOthers } from "#lib/utils/expandedFolders.js";
import FolderGroup from "./nav/FolderGroup.svelte";
import NavContextMenu from "./nav/NavContextMenu.svelte";
import NavToolbar from "./nav/NavToolbar.svelte";
import UncategorizedFeeds from "./nav/UncategorizedFeeds.svelte";

const DRAG_EXPAND_DELAY_MS = 600;

let expandTimeout: ReturnType<typeof setTimeout> | null = null;
let pendingFolderId: number | null = null;
let ctxMenu: NavContextMenu;
let folderListEl: HTMLDivElement | undefined;

const expanded = useExpandedFolders({
  get folders() {
    return navStore.folders;
  },
  get expandedFolders() {
    return navStore.expandedFolders;
  },
  set expandedFolders(v: Set<number>) {
    navStore.expandedFolders = v;
  },
});

$effect(() => {
  const feedId = navStore.selectedFeedId;
  const folderId = navStore.selectedFolderId;
  const folders = navStore.folders;

  if (folders.length > 0 && (feedId !== null || folderId !== null)) {
    requestAnimationFrame(() => {
      let selector: string | null = null;
      if (feedId !== null && feedId > 0) {
        selector = ".feed-item.selected";
      } else if (folderId !== null) {
        selector = ".folder-header.selected";
      }
      if (selector) {
        folderListEl?.querySelector<HTMLElement>(selector)?.scrollIntoView({ block: "nearest" });
      }
    });
  }
});

function cancelExpand() {
  if (expandTimeout) clearTimeout(expandTimeout);
  expandTimeout = null;
  pendingFolderId = null;
}

function handleNavDragOver(e: DragEvent) {
  e.preventDefault();

  const folderEl = (e.target as HTMLElement | null)?.closest?.("[data-folder-id]") as HTMLElement | null;
  const folderId = folderEl ? parseInt(folderEl.dataset.folderId ?? "", 10) : Number.NaN;
  if (Number.isNaN(folderId)) {
    cancelExpand();
    return;
  }
  if (pendingFolderId === folderId) return;

  cancelExpand();
  pendingFolderId = folderId;
  if (navStore.expandedFolders.has(folderId)) return;

  expandTimeout = setTimeout(() => {
    navStore.expandedFolders = expandFolderKeepingOthers(navStore.expandedFolders, folderId);
    expandTimeout = null;
    pendingFolderId = null;
  }, DRAG_EXPAND_DELAY_MS);
}

function onDragLeavePane(e: DragEvent) {
  const nav = e.currentTarget as HTMLElement;
  const related = e.relatedTarget as Node;
  if (!nav.contains(related)) cancelExpand();
}
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<nav
  class="pane"
  oncontextmenu={(e) => ctxMenu.show(e, "root", 0)}
  ondragover={handleNavDragOver}
  ondragleave={onDragLeavePane}
  ondrop={cancelExpand}
  ondragend={cancelExpand}
>
  <NavToolbar onExpandAll={expanded.expandAll} onCollapseAll={expanded.collapseAll} />

  <div class="folder-list" bind:this={folderListEl} onscroll={() => ctxMenu.close()}>
    {#each navStore.folders.filter((f) => f.id !== 0) as folder (folder.id)}
      <FolderGroup
        {folder}
        isExpanded={navStore.expandedFolders.has(folder.id)}
        onToggle={(e) => {
          e.stopPropagation();
          expanded.toggleFolder(folder.id);
        }}
        onContextMenu={(e, type, id, name) => ctxMenu.show(e, type, id, name)}
      />
    {/each}
    {#each navStore.folders.filter((f) => f.id === 0) as folder (folder.id)}
      <UncategorizedFeeds {folder} onContextMenu={(e, type, id, name) => ctxMenu.show(e, type, id, name)} />
    {/each}
  </div>

  <NavContextMenu bind:this={ctxMenu} />

  <div class="app-toolbar app-toolbar-bottom">
    <button type="button" onclick={() => uiStore.openSettings()} title="Settings" aria-label="Settings">
      <Settings size={18} />
    </button>
    <span class="footer-spacer"></span>
    <button type="button" onclick={() => uiStore.openAbout()} title="About FeedMee" aria-label="About FeedMee">
      <Info size={18} />
    </button>
  </div>
</nav>

<style>
.pane {
  background-color: var(--bg-pane);
  border-right: 1px solid var(--border-color);
  height: 100%;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  user-select: none;
}

.folder-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 1rem 1rem;
}

.footer-spacer {
  flex: 1;
}
</style>
