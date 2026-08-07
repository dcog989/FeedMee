<script lang="ts">
import { ChevronRight, RefreshCcwDot } from 'lucide-svelte';
import { flip } from 'svelte/animate';
import { tooltip } from '$lib/actions/tooltip.svelte';
import { feedStore, navStore, refreshStore } from '$lib/store.svelte';
import type { Feed, Folder } from '$lib/types';
import { createDragGhost } from '$lib/utils/dragGhost';
import FeedItem from './FeedItem.svelte';

let { folder, isExpanded, onToggle, onContextMenu, onFeedsChange } = $props<{
  folder: Folder;
  isExpanded: boolean;
  onToggle: (e: MouseEvent) => void;
  onContextMenu: (e: MouseEvent, type: 'folder' | 'feed', id: number, name?: string) => void;
  onFeedsChange: (folderId: number, feeds: Feed[]) => void;
}>();

const FLIP_DURATION = 200;

// --- Native DnD State ---
let dropIndex = $state<number | null>(null);
let unreadCount = $derived(getFolderUnreadCount(folder.feeds));

// --- Native DnD Handlers ---
function handleDragStart(e: DragEvent, feedId: number, feedName: string) {
  const dt = e.dataTransfer;
  if (!dt) return;
  dt.effectAllowed = 'move';
  dt.setData('text/plain', JSON.stringify({ feedId, folderId: folder.id }));
  createDragGhost(e, feedName);
}

function handleDragOver(e: DragEvent) {
  e.preventDefault();
  if (!isExpanded) return;

  const ul = e.currentTarget as HTMLUListElement;
  const items = Array.from(ul.children as HTMLCollectionOf<HTMLLIElement>);
  const mouseY = e.clientY;

  let idx = items.length;
  for (let i = 0; i < items.length; i++) {
    const rect = items[i].getBoundingClientRect();
    if (mouseY < rect.top + rect.height / 2) {
      idx = i;
      break;
    }
  }
  dropIndex = idx;
}

function handleDragLeaveList(e: DragEvent) {
  const ul = e.currentTarget as HTMLUListElement;
  const related = e.relatedTarget as Node;
  if (!ul.contains(related)) {
    dropIndex = null;
  }
}

function handleDrop(e: DragEvent) {
  e.preventDefault();
  const dt = e.dataTransfer;
  if (!dt) return;
  const data = dt.getData('text/plain');
  if (!data) return;

  const { feedId, folderId: sourceFolderId } = JSON.parse(data);

  const feeds = [...folder.feeds];
  const draggedIndex = feeds.findIndex((f) => f.id === feedId);

  if (draggedIndex !== -1) {
    const [item] = feeds.splice(draggedIndex, 1);
    const idx = dropIndex !== null ? (dropIndex > draggedIndex ? dropIndex - 1 : dropIndex) : feeds.length;
    feeds.splice(idx, 0, item);
    onFeedsChange(folder.id, feeds);
  } else {
    const allFeeds = navStore.folders.flatMap((f) => f.feeds);
    const feed = allFeeds.find((f) => f.id === feedId);
    if (!feed) return;
    const idx = dropIndex ?? feeds.length;
    feeds.splice(idx, 0, feed);
    onFeedsChange(folder.id, feeds);
  }

  if (sourceFolderId !== folder.id) {
    feedStore.moveFeed(feedId, folder.id);
  }

  dropIndex = null;
}

function handleDragEnd() {
  dropIndex = null;
}

function onHeaderDblClick(e: MouseEvent) {
  e.stopPropagation();
  onToggle(e);
}

function getFolderUnreadCount(feeds: Feed[]): number {
  return feeds.reduce((acc, feed) => acc + feed.unread_count, 0);
}
</script>

<div
  class="folder"
  role="treeitem"
  aria-expanded={isExpanded}
  aria-selected="false"
  tabindex="-1"
  data-folder-id={folder.id}
>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- biome-ignore lint/a11y/noStaticElementInteractions: oncontextmenu/ondblclick are mouse-only events with no keyboard analog -->
  <div
    class="folder-header"
    class:selected={navStore.selectedFolderId === folder.id}
    oncontextmenu={(e) => onContextMenu(e, 'folder', folder.id, folder.name)}
    ondblclick={onHeaderDblClick}
  >
    <button type="button" class="toggle-icon" onclick={onToggle} aria-label="Toggle folder">
      <ChevronRight size={10} style="transform: rotate({isExpanded ? 90 : 0}deg); transition: transform 0.2s;" />
    </button>

    <button
      type="button"
      class="folder-name-area"
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
          use:tooltip={refreshStore.isFolderFresh(folder.id)
                        ? 'Already fresh!'
                        : 'Click to refresh folder'}
          >{unreadCount}</span
        >
      {:else}
        <span
          class="refresh-icon folder-refresh"
          use:tooltip={refreshStore.isFolderFresh(folder.id)
                        ? 'Already fresh!'
                        : 'Click to refresh folder'}
        >
          <RefreshCcwDot size={16} />
        </span>
      {/if}
    </button>
  </div>

  <ul
    class="feed-list"
    class:collapsed={!isExpanded}
    class:drag-over={dropIndex !== null && isExpanded}
    ondragover={handleDragOver}
    ondragleave={handleDragLeaveList}
    ondrop={handleDrop}
  >
    {#if isExpanded}
      {#each folder.feeds as feed, i (feed.id)}
        <li
          animate:flip={{ duration: FLIP_DURATION }}
          draggable={true}
          ondragstart={(e) => handleDragStart(e, feed.id, feed.name)}
          ondragend={handleDragEnd}
          class:drop-before={dropIndex === i}
        >
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div
            class="feed-item"
            class:selected={navStore.selectedFeedId === feed.id}
            onclick={(e) => {
                            e.stopPropagation();
                            navStore.selectFeed(feed.id);
                        }}
            oncontextmenu={(e) => onContextMenu(e, 'feed', feed.id, feed.name)}
            role="option"
            tabindex="0"
            aria-selected={navStore.selectedFeedId === feed.id}
            onkeydown={(e) => {
                            if (e.key === 'Enter' || e.key === ' ') {
                                e.preventDefault();
                                navStore.selectFeed(feed.id);
                            }
                        }}
          >
            <FeedItem {feed} isSelected={navStore.selectedFeedId === feed.id} />
          </div>
        </li>
      {/each}
    {/if}
  </ul>
</div>

<style>
.folder {
  outline: none;
  margin-bottom: 2px;
  position: relative;
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
  list-style: none;
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

li.drop-before::before {
  content: "";
  position: absolute;
  top: -1px;
  left: 0;
  right: 0;
  height: 2px;
  background: var(--bg-selected);
  z-index: 5;
  pointer-events: none;
}

.feed-list li {
  position: relative;
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
