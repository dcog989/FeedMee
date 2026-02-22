<script lang="ts">
    import { tooltip } from '$lib/actions/tooltip.svelte';
    import { appState } from '$lib/store.svelte';
    import { dndState } from '$lib/dndState.svelte';
    import type { Feed, Folder } from '$lib/types';
    import { dndzone, TRIGGERS, type DndEvent, type Item } from 'svelte-dnd-action';
    import { flip } from 'svelte/animate';
    import { ChevronRight, X, RefreshCw } from 'lucide-svelte';

    let { folder, isExpanded, onToggle, onContextMenu, onExpandHover, onFeedsChange } = $props<{
        folder: Folder;
        isExpanded: boolean;
        onToggle: (e: MouseEvent) => void;
        onContextMenu: (e: MouseEvent, type: 'folder' | 'feed', id: number, name?: string) => void;
        onExpandHover: (id: number) => void;
        onFeedsChange: (folderId: number, feeds: Feed[]) => void;
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
        const feeds = e.detail.items as Feed[];
        onFeedsChange(folder.id, feeds);
        dndState.isDragging = true;
        if (!isExpanded && e.detail.info.trigger === TRIGGERS.DRAGGED_ENTERED) {
            onExpandHover(folder.id);
        }
    }

    function handleDndFinalize(e: CustomEvent<DndEvent<Item>>) {
        dndState.isDragging = false;
        const items = e.detail.items as Feed[];
        onFeedsChange(folder.id, items);
        items.forEach((feed) => {
            if (feed.folder_id !== folder.id) {
                appState.moveFeed(feed.id, folder.id);
            }
        });
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
        ondblclick={onHeaderDblClick}>
        <span class="toggle-icon" onclick={onToggle}>
            <ChevronRight
                size={10}
                style="transform: rotate({isExpanded ? 90 : 0}deg); transition: transform 0.2s;" />
        </span>

        <span
            class="folder-name-area"
            onclick={(e) => {
                appState.selectFolder(folder.id);
                onToggle(e);
            }}>
            <span class="folder-name">{folder.name}</span>

            {#if appState.isFolderUpdating(folder.id)}
                <div class="mini-spinner"></div>
            {:else if getFolderUnreadCount(folder.feeds) > 0}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <span
                    class="badge folder-badge"
                    onclick={(e) => {
                        e.stopPropagation();
                        appState.requestRefreshFolder(folder.id);
                    }}
                    use:tooltip={appState.isFolderFresh(folder.id)
                        ? 'Already fresh!'
                        : 'Click to refresh folder'}>{getFolderUnreadCount(folder.feeds)}</span>
            {/if}
        </span>
    </div>

    <ul
        class="feed-list"
        class:collapsed={!isExpanded}
        use:dndzone={{
            items: folder.feeds,
            flipDurationMs: FLIP_DURATION,
            type: 'feed',
            dropTargetStyle: isExpanded
                ? { outline: '2px solid var(--bg-selected)', borderRadius: '4px' }
                : {},
        }}
        onconsider={handleDndConsider}
        onfinalize={handleDndFinalize}>
        {#if isExpanded}
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
                        }}>
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
                                    <X size={10} color="white" />
                                </span>
                            {:else if feed.unread_count > 0}
                                <span
                                    class="badge"
                                    use:tooltip={appState.isFeedFresh(feed.id)
                                        ? 'Already fresh!'
                                        : 'Click to refresh'}>{feed.unread_count}</span>
                            {:else}
                                <span
                                    class="refresh-icon"
                                    use:tooltip={appState.isFeedFresh(feed.id)
                                        ? 'Already fresh!'
                                        : 'Refresh'}>
                                    <RefreshCw size={12} />
                                </span>
                            {/if}
                        </div>
                    </div>
                </li>
            {/each}
        {/if}
    </ul>
</div>

<style>
    /* Styles remain same as previous step, just applied to this block */
    .folder {
        outline: none;
        margin-bottom: 2px;
        position: relative;
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

    .feed-list.collapsed {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 32px;
        min-height: 0;
        overflow: hidden;
        padding: 0;
        opacity: 0;
        pointer-events: none;
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
