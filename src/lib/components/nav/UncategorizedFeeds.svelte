<script lang="ts">
import { flip } from 'svelte/animate';
import { feedStore, navStore, refreshStore } from '$lib/store.svelte';
import type { Folder } from '$lib/types';
import { createDragGhost } from '$lib/utils/dragGhost';
import FeedItem from './FeedItem.svelte';

let {
  folder,
  onContextMenu,
}: {
  folder: Folder;
  onContextMenu: (e: MouseEvent, type: 'feed', id: number, name?: string) => void;
} = $props();
</script>

{#if folder.id === 0}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <fieldset
    class="root-section"
    class:has-feeds={folder.feeds.length > 0}
    aria-label="Unfiled feeds"
    ondragover={(e) => e.preventDefault()}
    ondrop={(e) => {
            e.preventDefault();
            const dt = e.dataTransfer;
            if (!dt) return;
            const data = dt.getData('text/plain');
            if (!data) return;
            const { feedId } = JSON.parse(data);
            feedStore.moveFeed(feedId, null);
        }}
  >
    <div class="root-header">UNCATEGORIZED</div>
    {#each folder.feeds as feed (feed.id)}
      <div
        animate:flip={{ duration: 200 }}
        class="feed-item"
        class:selected={navStore.selectedFeedId === feed.id}
        onclick={(e) => {
                    e.stopPropagation();
                    navStore.selectFeed(feed.id);
                }}
        oncontextmenu={(e) => onContextMenu(e, 'feed', feed.id, feed.name)}
        draggable={true}
        ondragstart={(e) => {
                    const dt = e.dataTransfer;
                    if (!dt) return;
                    dt.effectAllowed = 'move';
                    dt.setData('text/plain', JSON.stringify({ feedId: feed.id, folderId: 0 }));
                    createDragGhost(e, feed.name);
                }}
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
    {/each}
  </fieldset>
{/if}

<style>
.root-section {
  margin: 0;
  padding: 0 0 4px;
  border: none;
  min-inline-size: 0;
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
