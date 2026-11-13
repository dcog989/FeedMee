<script lang="ts">
import { flip } from "svelte/animate";
import { feedStore, navStore } from "#lib/store.svelte.js";
import type { Folder } from "#lib/types.js";
import { parseDragData, startFeedDrag } from "#lib/utils/dragGhost.js";
import FeedItem from "./FeedItem.svelte";

let {
  folder,
  onContextMenu,
}: {
  folder: Folder;
  onContextMenu: (e: MouseEvent, type: "feed", id: number, name?: string) => void;
} = $props();

function handleDrop(e: DragEvent) {
  e.preventDefault();
  const data = e.dataTransfer?.getData("text/plain");
  if (!data) return;
  const payload = parseDragData(data);
  if (!payload || payload.folderId === folder.id) return;
  feedStore.moveFeed(payload.feedId, null);
}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="root-section"
  class:has-feeds={folder.feeds.length > 0}
  role="listbox"
  aria-label="Unfiled feeds"
  ondragover={(e) => e.preventDefault()}
  ondrop={handleDrop}
>
  <div class="root-header" aria-hidden="true">UNCATEGORIZED</div>
  {#each folder.feeds as feed (feed.id)}
    <div
      animate:flip={{ duration: 200 }}
      class="feed-item"
      class:selected={navStore.selectedFeedId === feed.id}
      onclick={(e) => {
        e.stopPropagation();
        navStore.selectFeed(feed.id);
      }}
      oncontextmenu={(e) => onContextMenu(e, "feed", feed.id, feed.name)}
      draggable={true}
      ondragstart={(e) => startFeedDrag(e, feed, folder.id)}
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
</div>

<style>
.root-section {
  margin: 0;
  padding: 0 0 4px;
}

.root-section.has-feeds {
  border-top: 2px solid var(--border-color);
  margin-top: 8px;
  padding-top: 4px;
}

.root-header {
  font-size: 0.75rem;
  text-transform: uppercase;
  font-weight: 700;
  letter-spacing: 0.5px;
  color: var(--text-secondary);
  padding: 4px 0.6rem 4px 4px;
  cursor: default;
}
</style>
