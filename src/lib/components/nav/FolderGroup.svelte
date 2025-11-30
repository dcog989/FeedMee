<script lang="ts">
    import { appState } from "$lib/store.svelte";
    import type { Feed, Folder } from "$lib/types";
    import { dndzone, type DndEvent } from "svelte-dnd-action";
    import { flip } from "svelte/animate";

    let { folder, isExpanded, onToggle, onContextMenu, onExpandHover } = $props<{
        folder: Folder;
        isExpanded: boolean;
        onToggle: (e: MouseEvent) => void;
        onContextMenu: (e: MouseEvent, type: "folder" | "feed", id: number, name?: string) => void;
        onExpandHover: (id: number) => void;
    }>();

    const FLIP_DURATION = 200;

    function getFavicon(url: string) {
        try {
            const domain = new URL(url).hostname;
            return `https://www.google.com/s2/favicons?domain=${domain}&sz=32`;
        } catch {
            return "";
        }
    }

    function getFolderUnreadCount(feeds: Feed[]): number {
        return feeds.reduce((acc, feed) => acc + feed.unread_count, 0);
    }

    // --- DnD List Handlers ---
    function handleDndConsider(e: CustomEvent<DndEvent<any>>) {
        folder.feeds = e.detail.items as Feed[];
    }

    function handleDndFinalize(e: CustomEvent<DndEvent<any>>) {
        folder.feeds = e.detail.items as Feed[];
        const items = e.detail.items as Feed[];
        items.forEach((feed) => {
            if (feed.folder_id !== folder.id) {
                feed.folder_id = folder.id;
                appState.moveFeed(feed.id, folder.id);
            }
        });
    }

    // --- Drag Start (Feed) ---
    function onFeedDragStart(e: DragEvent, feedId: number) {
        if (e.dataTransfer) {
            e.dataTransfer.setData("text/plain", feedId.toString());
            e.dataTransfer.effectAllowed = "move";
        }
    }

    // --- Drop on Folder Header ---
    function onHeaderDragOver(e: DragEvent) {
        e.preventDefault();
        e.stopPropagation();
        if (e.dataTransfer) {
            e.dataTransfer.dropEffect = "move";
        }
    }

    function onHeaderDrop(e: DragEvent) {
        e.preventDefault();
        e.stopPropagation();
        const data = e.dataTransfer?.getData("text/plain");
        if (data) {
            const feedId = parseInt(data);
            if (!isNaN(feedId) && feedId > 0) {
                appState.moveFeed(feedId, folder.id);
            }
        }
    }
</script>

<div class="folder" role="treeitem" aria-expanded={isExpanded} aria-selected="false" tabindex="-1">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="folder-header" onclick={onToggle} oncontextmenu={(e) => onContextMenu(e, "folder", folder.id, folder.name)} ondragenter={() => onExpandHover(folder.id)} ondragover={onHeaderDragOver} ondrop={onHeaderDrop}>
        <span class="toggle-icon">
            <svg width="10" height="10" viewBox="0 0 10 10" style="transform: rotate({isExpanded ? 90 : 0}deg); transition: transform 0.2s;">
                <path d="M2,2 L8,5 L2,8" fill="currentColor" />
            </svg>
        </span>
        <span class="folder-name">{folder.name}</span>
        {#if getFolderUnreadCount(folder.feeds) > 0}
            <span class="badge folder-badge">{getFolderUnreadCount(folder.feeds)}</span>
        {/if}
    </div>

    {#if isExpanded}
        <ul class="feed-list" use:dndzone={{ items: folder.feeds, flipDurationMs: FLIP_DURATION, type: "feed", dropTargetStyle: { outline: "2px solid var(--bg-selected)", borderRadius: "4px" } }} onconsider={handleDndConsider} onfinalize={handleDndFinalize}>
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
                        oncontextmenu={(e) => onContextMenu(e, "feed", feed.id)}
                        role="option"
                        tabindex="0"
                        aria-selected={appState.selectedFeedId === feed.id}
                        onkeydown={(e) => {
                            if (e.key === "Enter" || e.key === " ") {
                                e.preventDefault();
                                appState.selectFeed(feed.id);
                            }
                        }}
                        draggable="true"
                        ondragstart={(e) => onFeedDragStart(e, feed.id)}
                    >
                        <span class="feed-name-wrap">
                            {#if feed.url}
                                <img src={getFavicon(feed.url)} alt="" class="feed-favicon" loading="lazy" />
                            {:else}
                                <span class="feed-icon">#</span>
                            {/if}
                            <span class="feed-name">{feed.name}</span>
                        </span>

                        {#if feed.unread_count > 0}
                            <span class="badge">{feed.unread_count}</span>
                        {/if}
                    </div>
                </li>
            {/each}
        </ul>
    {/if}
</div>

<style>
    .folder {
        outline: none;
        margin-bottom: 2px;
    }

    .folder-header {
        display: flex;
        align-items: center;
        padding: 6px 4px;
        cursor: pointer;
        color: var(--text-secondary);
        border-radius: 4px;
        transition: background-color 0.2s;
    }

    /* Ensure events pass to the header for dropping */
    .folder-header > * {
        pointer-events: none;
    }

    .folder-header:hover {
        color: var(--text-primary);
        background-color: rgba(0, 0, 0, 0.03);
    }

    .toggle-icon {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 16px;
        height: 16px;
    }

    .folder-name {
        font-size: 0.75rem;
        text-transform: uppercase;
        font-weight: 700;
        letter-spacing: 0.5px;
        margin-left: 4px;
        flex: 1;
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

    .folder-badge {
        opacity: 0.7;
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
</style>
