<script lang="ts">
    import { tooltip } from '$lib/actions/tooltip.svelte';
    import { appState } from '$lib/store.svelte';
    import type { Feed, Folder } from '$lib/types';
    import { dndzone, type DndEvent, type Item } from 'svelte-dnd-action';
    import { flip } from 'svelte/animate';
    import { slide } from 'svelte/transition';

    let { folder, isExpanded, onToggle, onContextMenu, onExpandHover } = $props<{
        folder: Folder;
        isExpanded: boolean;
        onToggle: (e: MouseEvent) => void;
        onContextMenu: (e: MouseEvent, type: 'folder' | 'feed', id: number, name?: string) => void;
        onExpandHover: (id: number) => void;
    }>();

    const FLIP_DURATION = 200;

    function getFavicon(url: string) {
        try {
            const domain = new URL(url).hostname;
            return `https://www.google.com/s2/favicons?domain=${domain}&sz=32`;
        } catch {
            return '';
        }
    }

    function getFolderUnreadCount(feeds: Feed[]): number {
        return feeds.reduce((acc, feed) => acc + feed.unread_count, 0);
    }

    // --- DnD List Handlers ---
    function handleDndConsider(e: CustomEvent<DndEvent<Item>>) {
        folder.feeds = e.detail.items as Feed[];
    }

    function handleDndFinalize(e: CustomEvent<DndEvent<Item>>) {
        folder.feeds = e.detail.items as Feed[];
        const items = e.detail.items as Feed[];
        items.forEach((feed) => {
            if (feed.folder_id !== folder.id) {
                feed.folder_id = folder.id;
                appState.moveFeed(feed.id, folder.id);
            }
        });
    }

    function onFeedDragStart(e: DragEvent, feedId: number) {
        if (e.dataTransfer) {
            e.dataTransfer.setData('text/plain', feedId.toString());
            e.dataTransfer.effectAllowed = 'move';
        }
    }

    function onHeaderDragOver(e: DragEvent) {
        e.preventDefault();
        e.stopPropagation();
        if (e.dataTransfer) {
            e.dataTransfer.dropEffect = 'move';
        }
        onExpandHover(folder.id);
    }

    function onHeaderDrop(e: DragEvent) {
        e.preventDefault();
        e.stopPropagation();
        const data = e.dataTransfer?.getData('text/plain');
        if (data) {
            const feedId = parseInt(data);
            if (!isNaN(feedId) && feedId > 0) {
                appState.moveFeed(feedId, folder.id);
            }
        }
    }

    function onHeaderDblClick(e: MouseEvent) {
        e.stopPropagation();
        onToggle(e);
    }
</script>

<div class="folder" role="treeitem" aria-expanded={isExpanded} aria-selected="false" tabindex="-1">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
        class="folder-header"
        class:selected={appState.selectedFolderId === folder.id}
        oncontextmenu={(e) => onContextMenu(e, 'folder', folder.id, folder.name)}
        ondragover={onHeaderDragOver}
        ondrop={onHeaderDrop}
        ondblclick={onHeaderDblClick}>
        <span class="toggle-icon" onclick={onToggle}>
            <svg
                width="10"
                height="10"
                viewBox="0 0 10 10"
                style="transform: rotate({isExpanded ? 90 : 0}deg); transition: transform 0.2s;">
                <path d="M2,2 L8,5 L2,8" fill="currentColor" />
            </svg>
        </span>

        <span class="folder-name-area" onclick={() => appState.selectFolder(folder.id)}>
            <span class="folder-name">{folder.name}</span>

            {#if appState.isFolderUpdating(folder.id)}
                <div class="mini-spinner"></div>
            {:else if getFolderUnreadCount(folder.feeds) > 0}
                <span class="badge folder-badge">{getFolderUnreadCount(folder.feeds)}</span>
            {/if}
        </span>
    </div>

    {#if isExpanded}
        <ul
            class="feed-list"
            transition:slide={{ duration: 200 }}
            use:dndzone={{
                items: folder.feeds,
                flipDurationMs: FLIP_DURATION,
                type: 'feed',
                dropTargetStyle: { outline: '2px solid var(--bg-selected)', borderRadius: '4px' },
            }}
            onconsider={handleDndConsider}
            onfinalize={handleDndFinalize}>
            {#each folder.feeds as feed (feed.id)}
                <li animate:flip={{ duration: FLIP_DURATION }}>
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <div
                        class="feed-item"
                        class:selected={appState.selectedFeedId === feed.id}
                        onclick={(e) => {
                            e.stopPropagation();
                            appState.selectFeed(feed.id);
                        }}
                        oncontextmenu={(e) => onContextMenu(e, 'feed', feed.id)}
                        role="option"
                        tabindex="0"
                        aria-selected={appState.selectedFeedId === feed.id}
                        onkeydown={(e) => {
                            if (e.key === 'Enter' || e.key === ' ') {
                                e.preventDefault();
                                appState.selectFeed(feed.id);
                            }
                        }}
                        draggable="true"
                        ondragstart={(e) => onFeedDragStart(e, feed.id)}>
                        <span class="feed-name-wrap">
                            {#if feed.url}
                                <img
                                    src={getFavicon(feed.url)}
                                    alt=""
                                    class="feed-favicon"
                                    loading="lazy" />
                            {:else}
                                <span class="feed-icon">#</span>
                            {/if}
                            <span class="feed-name">{feed.name}</span>
                        </span>

                        <!-- Action Area -->
                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <div
                            class="feed-action-area"
                            onclick={(e) => {
                                e.stopPropagation();
                                appState.requestRefreshFeed(feed.id);
                            }}>
                            {#if appState.isFeedUpdating(feed.id)}
                                <div class="mini-spinner"></div>
                            {:else if feed.has_error}
                                <span class="error-badge" use:tooltip={'Feed update failed'}>
                                    <svg width="10" height="10" viewBox="0 0 10 10">
                                        <line
                                            x1="2"
                                            y1="2"
                                            x2="8"
                                            y2="8"
                                            stroke="white"
                                            stroke-width="2" />
                                        <line
                                            x1="8"
                                            y1="2"
                                            x2="2"
                                            y2="8"
                                            stroke="white"
                                            stroke-width="2" />
                                    </svg>
                                </span>
                            {:else if feed.unread_count > 0}
                                <span class="badge" use:tooltip={'Click to refresh'}
                                    >{feed.unread_count}</span>
                            {:else}
                                <span class="refresh-icon" use:tooltip={'Refresh'}>
                                    <svg
                                        width="12"
                                        height="12"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2">
                                        <path d="M23 4v6h-6"></path>
                                        <path d="M1 20v-6h6"></path>
                                        <path
                                            d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"
                                        ></path>
                                    </svg>
                                </span>
                            {/if}
                        </div>
                    </div>
                </li>
            {/each}
        </ul>
    {/if}
</div>

<style>
    /* Styles remain same as previous step, just applied to this block */
    .folder {
        outline: none;
        margin-bottom: 2px;
    }

    .folder-header {
        display: flex;
        align-items: center;
        padding: 4px 4px;
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

    .feed-item {
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

    .feed-name-wrap {
        display: flex;
        align-items: center;
        gap: 8px;
        white-space: nowrap;
        overflow: hidden;
    }

    .feed-name {
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .feed-icon {
        color: var(--text-secondary);
        font-size: 0.8rem;
        opacity: 0.7;
        flex-shrink: 0;
    }

    .feed-favicon {
        width: 16px;
        height: 16px;
        border-radius: 2px;
    }

    .feed-action-area {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        min-width: 24px;
        height: 100%;
        cursor: pointer;
        padding-left: 8px;
    }

    .badge {
        background-color: var(--text-secondary);
        color: var(--bg-pane);
        font-size: 0.75rem;
        padding: 1px 6px;
        border-radius: 10px;
        font-weight: 600;
        min-width: 16px;
        text-align: center;
        flex-shrink: 0;
    }

    .badge:hover {
        background-color: var(--bg-selected);
        color: white;
    }

    .error-badge {
        width: 16px;
        height: 16px;
        background-color: #d32f2f;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
    }

    .folder-badge {
        opacity: 0.7;
    }

    .refresh-icon {
        color: var(--text-secondary);
        opacity: 0.5;
        transition: opacity 0.2s;
        display: flex;
        align-items: center;
    }

    .refresh-icon:hover {
        opacity: 1;
        color: var(--text-primary);
    }

    .feed-item:hover {
        background-color: var(--bg-hover);
    }

    .feed-item.selected {
        background-color: var(--bg-selected-muted);
        color: var(--text-primary);
        border-left-color: var(--bg-selected);
        font-weight: 500;
    }

    .feed-item.selected .feed-icon {
        color: var(--bg-selected);
    }

    .feed-item.selected .badge {
        background-color: var(--bg-selected);
        color: white;
    }

    .mini-spinner {
        width: 12px;
        height: 12px;
        border: 2px solid var(--text-secondary);
        border-top-color: transparent;
        border-radius: 50%;
        animation: spin 1s linear infinite;
        flex-shrink: 0;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }
</style>
