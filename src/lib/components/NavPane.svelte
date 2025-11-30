<script lang="ts">
    import { appState } from "$lib/store.svelte";
    import type { Feed } from "$lib/types";
    import { dndzone, type DndEvent } from "svelte-dnd-action";
    import { flip } from "svelte/animate";

    let expandedFolders = $state<Set<number>>(new Set());
    let initialized = false;

    // Context Menu State
    let cmVisible = $state(false);
    let cmX = $state(0);
    let cmY = $state(0);
    let cmTarget = $state<{ type: "folder" | "feed"; id: number; name?: string } | null>(null);

    const FLIP_DURATION = 200;

    $effect(() => {
        if (!initialized && appState.folders.length > 0) {
            const newSet = new Set(expandedFolders);
            appState.folders.forEach((f) => newSet.add(f.id));
            expandedFolders = newSet;
            initialized = true;
        }
    });

    function toggleFolder(id: number, e: MouseEvent) {
        e.stopPropagation();
        const newSet = new Set(expandedFolders);
        if (newSet.has(id)) {
            newSet.delete(id);
        } else {
            newSet.add(id);
        }
        expandedFolders = newSet;
    }

    // --- Drag & Drop Handlers ---

    function handleDndConsider(folderId: number, e: CustomEvent<DndEvent<Feed>>) {
        const folder = appState.folders.find((f) => f.id === folderId);
        if (folder) {
            folder.feeds = e.detail.items;
        }
    }

    function handleDndFinalize(folderId: number, e: CustomEvent<DndEvent<Feed>>) {
        const folder = appState.folders.find((f) => f.id === folderId);
        if (folder) {
            folder.feeds = e.detail.items;

            // Check for items that were moved into this folder
            e.detail.items.forEach((feed) => {
                if (feed.folder_id !== folderId) {
                    // Update Local State Optimistically
                    feed.folder_id = folderId;
                    // Persist to DB
                    appState.moveFeed(feed.id, folderId);
                }
            });
        }
    }

    // --- Context Menu ---
    function handleContextMenu(event: MouseEvent, type: "folder" | "feed", id: number, name?: string) {
        event.preventDefault();
        event.stopPropagation();
        cmVisible = true;
        cmX = event.clientX;
        cmY = event.clientY;
        cmTarget = { type, id, name };
    }

    function closeContextMenu() {
        cmVisible = false;
        cmTarget = null;
    }

    function cmRename() {
        if (!cmTarget || cmTarget.type !== "folder") return;
        const newName = prompt("Rename Folder:", cmTarget.name);
        if (newName && newName.trim() !== "") {
            appState.renameFolder(cmTarget.id, newName.trim());
        }
        closeContextMenu();
    }

    function cmDelete() {
        if (!cmTarget) return;
        if (cmTarget.type === "folder") {
            appState.deleteFolder(cmTarget.id);
        } else {
            appState.deleteFeed(cmTarget.id);
        }
        closeContextMenu();
    }
</script>

<svelte:window onclick={closeContextMenu} />

<nav class="pane">
    <div class="folder-list" role="tree">
        {#each appState.folders as folder (folder.id)}
            <div class="folder" role="treeitem" aria-expanded={expandedFolders.has(folder.id)} aria-selected="false" tabindex="-1">
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="folder-header" onclick={(e) => toggleFolder(folder.id, e)} oncontextmenu={(e) => handleContextMenu(e, "folder", folder.id, folder.name)}>
                    <span class="toggle-icon">
                        <svg width="10" height="10" viewBox="0 0 10 10" style="transform: rotate({expandedFolders.has(folder.id) ? 90 : 0}deg); transition: transform 0.2s;">
                            <path d="M2,2 L8,5 L2,8" fill="currentColor" />
                        </svg>
                    </span>
                    <span class="folder-name">{folder.name}</span>
                </div>

                {#if expandedFolders.has(folder.id)}
                    <ul class="feed-list" use:dndzone={{ items: folder.feeds, flipDurationMs: FLIP_DURATION, type: "feed", dropTargetStyle: { outline: "2px solid var(--bg-selected)", borderRadius: "4px" } }} onconsider={(e) => handleDndConsider(folder.id, e)} onfinalize={(e) => handleDndFinalize(folder.id, e)}>
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
                                    oncontextmenu={(e) => handleContextMenu(e, "feed", feed.id)}
                                    role="option"
                                    tabindex="0"
                                    aria-selected={appState.selectedFeedId === feed.id}
                                    onkeydown={(e) => {
                                        if (e.key === "Enter" || e.key === " ") {
                                            e.preventDefault();
                                            appState.selectFeed(feed.id);
                                        }
                                    }}
                                >
                                    <span class="feed-name-wrap">
                                        <span class="feed-icon">#</span>
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
        {/each}
    </div>

    {#if cmVisible}
        <div class="context-menu" style="top: {cmY}px; left: {cmX}px">
            {#if cmTarget?.type === "folder"}
                <button onclick={cmRename}>Rename Folder</button>
            {/if}
            <button class="danger" onclick={cmDelete}>Delete</button>
        </div>
    {/if}
</nav>

<style>
    .pane {
        background-color: var(--bg-pane);
        border-right: 1px solid var(--border-color);
        height: 100%;
        display: flex;
        flex-direction: column;
        box-sizing: border-box;
        padding-top: 0.5rem;
        user-select: none;
    }

    .folder-list {
        flex: 1;
        overflow-y: auto;
        padding: 0 1rem 1rem;
    }

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
    }

    .feed-list {
        list-style: none;
        padding: 0 0 0 20px;
        margin: 0;
        /* Ensure empty folders can receive drops */
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
        /* Important for DnD styling */
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

    .context-menu {
        position: fixed;
        background: var(--bg-app);
        border: 1px solid var(--border-color);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
        border-radius: 6px;
        padding: 4px;
        z-index: 1000;
        min-width: 120px;
    }

    .context-menu button {
        display: block;
        width: 100%;
        text-align: left;
        background: none;
        border: none;
        padding: 8px 12px;
        cursor: pointer;
        color: var(--text-primary);
        border-radius: 4px;
        font-size: 0.9rem;
    }

    .context-menu button:hover {
        background-color: var(--bg-hover);
    }

    .context-menu button.danger {
        color: #e81123;
    }
    .context-menu button.danger:hover {
        background-color: #ffeef0;
    }
</style>
