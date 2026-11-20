<script lang="ts">
import { ChevronDown, ChevronUp, FolderPlus, Info, RefreshCw, Rss, Settings } from "@lucide/svelte";
import { tooltip } from "#lib/actions/tooltip.svelte.js";
import { refreshStore, uiStore } from "#lib/store.svelte.js";

let { onExpandAll, onCollapseAll }: { onExpandAll: () => void; onCollapseAll: () => void } = $props();

let isRefreshing = $derived(refreshStore.isRefreshingFeeds);

function createFolder() {
  uiStore.showNewFolderDialog = true;
}

function manageContent() {
  uiStore.showAddDialog = true;
}
</script>

<div class="app-toolbar">
  <button type="button" onclick={onExpandAll} use:tooltip={"Expand All"} aria-label="Expand All">
    <ChevronDown size={18} />
  </button>
  <button type="button" onclick={onCollapseAll} use:tooltip={"Collapse All"} aria-label="Collapse All">
    <ChevronUp size={18} />
  </button>
  <button
    type="button"
    onclick={() => refreshStore.refreshAllFeeds()}
    use:tooltip={() => (refreshStore.isAllFresh() ? "Already fresh!" : "Refresh All")}
    disabled={isRefreshing}
    aria-label="Refresh All"
  >
    <RefreshCw size={18} class={isRefreshing ? "spinning" : ""} />
  </button>
  <button type="button" onclick={createFolder} use:tooltip={"New Folder"} aria-label="New Folder">
    <FolderPlus size={18} />
  </button>
  <button type="button" onclick={manageContent} use:tooltip={"Manage Content"} aria-label="Manage Content">
    <Rss size={18} />
  </button>
  <span class="toolbar-spacer"></span>
  <button type="button" onclick={() => uiStore.openSettings()} use:tooltip={"Settings"} aria-label="Settings">
    <Settings size={18} />
  </button>
  <button type="button" onclick={() => uiStore.openAbout()} use:tooltip={"About FeedMee"} aria-label="About FeedMee">
    <Info size={18} />
  </button>
</div>

<style>
.toolbar-spacer {
  flex: 1;
}

:global(.spinning) {
  animation: spin 1s linear infinite;
}
</style>
