<script lang="ts">
import { ChevronRight, RefreshCcwDot } from "@lucide/svelte";
import { flip } from "svelte/animate";
import { tooltip } from "#lib/actions/tooltip.svelte.js";
import { feedStore, navStore, refreshStore } from "#lib/store.svelte.js";
import type { Feed, Folder } from "#lib/types.js";
import { parseDragData, startFeedDrag } from "#lib/utils/dragGhost.js";
import FeedItem from "./FeedItem.svelte";

let {
  folder,
  isExpanded,
  onToggle,
  onContextMenu,
}: {
  folder: Folder;
  isExpanded: boolean;
  onToggle: (e: MouseEvent) => void;
  onContextMenu: (e: MouseEvent, type: "folder" | "feed", id: number, name?: string) => void;
} = $props();

const FLIP_DURATION = 200;

// --- Native DnD State ---
let isDragOver = $state(false);
let unreadCount = $derived(getFolderUnreadCount(folder.feeds));

// --- Native DnD Handlers ---
function handleDragOver(e: DragEvent) {
  e.preventDefault();
  if (!isExpanded) return;
  isDragOver = true;
}

function handleDragLeaveList(e: DragEvent) {
  const ul = e.currentTarget as HTMLUListElement;
  if (!ul.contains(e.relatedTarget as Node)) {
    isDragOver = false;
  }
}

// A drop only moves the feed; feed order is alphabetical and owned by the backend.
function handleDrop(e: DragEvent) {
  e.preventDefault();
  isDragOver = false;
  const data = e.dataTransfer?.getData("text/plain");
  if (!data) return;

  const payload = parseDragData(data);
  if (!payload || payload.folderId === folder.id) return;
  feedStore.moveFeed(payload.feedId, folder.id);
}

function handleDragEnd() {
  isDragOver = false;
}

function getFolderUnreadCount(feeds: Feed[]): number {
  return feeds.reduce((acc, feed) => acc + feed.unread_count, 0);
}
</script>

<div class="folder" data-folder-id={folder.id}>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- biome-ignore lint/a11y/noStaticElementInteractions: oncontextmenu is a mouse-only event with no keyboard analog -->
  <div
    class="folder-header"
    class:selected={navStore.selectedFolderId === folder.id}
    oncontextmenu={(e) => onContextMenu(e, "folder", folder.id, folder.name)}
  >
    <button type="button" class="toggle-icon" onclick={onToggle} aria-expanded={isExpanded} aria-label="Toggle folder">
      <ChevronRight size={10} style="transform: rotate({isExpanded ? 90 : 0}deg); transition: transform 0.2s;" />
    </button>

    <button
      type="button"
      class="folder-name-area"
      aria-current={navStore.selectedFolderId === folder.id}
      onclick={(e) => {
        navStore.selectFolder(folder.id);
        onToggle(e);
      }}
    >
      <span class="folder-name">{folder.name}</span>
    </button>

    <button
      type="button"
      class="folder-action-area"
      onclick={(e) => {
        e.stopPropagation();
        refreshStore.requestRefreshFolder(folder.id);
      }}
      aria-label="Refresh folder"
    >
      {#if refreshStore.isFolderUpdating(folder.id)}
        <div class="mini-spinner"></div>
      {:else if unreadCount > 0}
        <span
          class="badge folder-badge"
          use:tooltip={() => (refreshStore.isFolderFresh(folder.id) ? "Already fresh!" : "Click to refresh folder")}
          >{unreadCount}</span
        >
      {:else}
        <span
          class="refresh-icon folder-refresh"
          use:tooltip={() => (refreshStore.isFolderFresh(folder.id) ? "Already fresh!" : "Click to refresh folder")}
        >
          <RefreshCcwDot size={16} />
        </span>
      {/if}
    </button>
  </div>

  <div
    class="feed-list"
    class:collapsed={!isExpanded}
    class:drag-over={isDragOver && isExpanded}
    role="listbox"
    tabindex="-1"
    aria-label={folder.name}
    ondragover={handleDragOver}
    ondragleave={handleDragLeaveList}
    ondrop={handleDrop}
  >
    {#if isExpanded}
      {#each folder.feeds as feed (feed.id)}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          class="feed-item"
          animate:flip={{ duration: FLIP_DURATION }}
          class:selected={navStore.selectedFeedId === feed.id}
          draggable={true}
          ondragstart={(e) => startFeedDrag(e, feed, folder.id)}
          ondragend={handleDragEnd}
          onclick={(e) => {
            e.stopPropagation();
            navStore.selectFeed(feed.id);
          }}
          oncontextmenu={(e) => onContextMenu(e, "feed", feed.id, feed.name)}
          role="option"
          tabindex="0"
          aria-selected={navStore.selectedFeedId === feed.id}
          onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              navStore.selectFeed(feed.id);
            }
          }}
        >
          <FeedItem {feed} isSelected={navStore.selectedFeedId === feed.id} />
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
.folder {
  outline: none;
  margin-bottom: 2px;
}

.folder-header {
  display: flex;
  align-items: center;
  padding: 4px 0.6rem 4px 4px;
  border-left: 3px solid transparent;
  box-sizing: border-box;
  width: 100%;
  cursor: default;
  color: var(--text-secondary);
  border-radius: 4px;
  transition: background-color 0.2s;
  position: relative;
}

.folder-header:hover {
  color: var(--text-primary);
  background-color: rgba(0, 0, 0, 0.03);
}

.folder-header.selected {
  background-color: var(--bg-selected-muted);
  color: var(--text-primary);
}

.toggle-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  cursor: pointer;
  opacity: 0.7;
  border: none;
  background: transparent;
  padding: 0;
  font: inherit;
  color: inherit;
}

.toggle-icon:hover {
  opacity: 1;
}

.folder-name-area {
  flex: 1;
  display: flex;
  align-items: center;
  cursor: pointer;
  padding: 2px 0;
  overflow: hidden;
  border: none;
  background: transparent;
  font: inherit;
  color: inherit;
  text-align: left;
}

.folder-name {
  font-size: 0.75rem;
  text-transform: uppercase;
  font-weight: 700;
  letter-spacing: 0.5px;
  margin-left: 2px;
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.feed-list {
  padding: 0 0 0 20px;
  margin: 0;
  min-height: 10px;
}

.feed-list.collapsed {
  height: 0;
  min-height: 0;
  overflow: hidden;
  padding: 0;
  margin: 0;
  opacity: 0;
}

.feed-list.drag-over {
  outline: 2px solid var(--bg-selected);
  border-radius: 4px;
}

.folder-badge {
  opacity: 0.7;
}

.folder-action-area {
  display: flex;
  align-items: center;
  min-width: 24px;
  margin-left: auto;
  padding-left: 8px;
  justify-content: center;
  cursor: pointer;
  border: none;
  background: transparent;
  font: inherit;
  color: inherit;
}

.folder-refresh {
  opacity: 0.5;
}
</style>
