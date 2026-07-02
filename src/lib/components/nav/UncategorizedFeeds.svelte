<script lang="ts">
import { flip } from 'svelte/animate';
import { appState } from '$lib/store.svelte';
import type { Folder } from '$lib/types';
import FeedItem from './FeedItem.svelte';

let { folder, onContextMenu }: {
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
            appState.moveFeed(feedId, null);
        }}
    >
        <div class="root-header">UNCATEGORIZED</div>
        {#each folder.feeds as feed (feed.id)}
            <div
                animate:flip={{ duration: 200 }}
                class="feed-item"
                class:selected={appState.selectedFeedId === feed.id}
                onclick={(e) => {
                    e.stopPropagation();
                    appState.selectFeed(feed.id);
                }}
                oncontextmenu={(e) => onContextMenu(e, 'feed', feed.id, feed.name)}
                draggable={true}
                ondragstart={(e) => {
                    const dt = e.dataTransfer;
                    if (!dt) return;
                    dt.effectAllowed = 'move';
                    dt.setData('text/plain', JSON.stringify({ feedId: feed.id, folderId: 0 }));
                    const root = document.documentElement;
                    const style = getComputedStyle(root);
                    const bg = style.getPropertyValue('--bg-content').trim() || '#333';
                    const text = style.getPropertyValue('--text-primary').trim() || '#fff';
                    const pink = style.getPropertyValue('--bg-selected').trim() || '#ec4899';
                    const img = document.createElement('div');
                    img.textContent = feed.name;
                    img.style.cssText = `padding:2px 8px;background:${bg};color:${text};border:1px solid ${pink};border-radius:4px;font:8px/1.3 sans-serif;white-space:nowrap;position:absolute;top:-1000px;left:-1000px;pointer-events:none;`;
                    document.body.appendChild(img);
                    dt.setDragImage(img, 0, 0);
                    requestAnimationFrame(() => document.body.removeChild(img));
                }}
                role="option"
                tabindex="0"
                aria-selected={appState.selectedFeedId === feed.id}
                onkeydown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault();
                        appState.selectFeed(feed.id);
                    }
                }}
            >
                <FeedItem {feed} isSelected={appState.selectedFeedId === feed.id} />
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

.root-section .feed-item {
    width: 100%;
    padding: 0.4rem 0.6rem;
    background: transparent;
    text-align: left;
    cursor: pointer;
    border-radius: 6px;
    font-size: 0.9rem;
    color: var(--text-primary);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    border-left: 3px solid transparent;
    box-sizing: border-box;
}

.root-section .feed-item:hover {
    background-color: var(--bg-hover);
}

.root-section .feed-item.selected {
    background-color: var(--bg-selected-muted);
    color: var(--text-primary);
    border-left-color: var(--bg-selected);
    font-weight: 500;
}
</style>
