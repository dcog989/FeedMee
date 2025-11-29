<script lang="ts">
    import { appState } from "$lib/store.svelte";

    let expandedFolders = $state<Set<number>>(new Set());

    // Context Menu State
    let cmVisible = $state(false);
    let cmX = $state(0);
    let cmY = $state(0);
    let cmTarget = $state<{ type: "folder" | "feed"; id: number; name?: string } | null>(null);

    $effect(() => {
        // Auto-expand all folders on initial load if empty
        if (appState.folders.length > 0 && expandedFolders.size === 0) {
            appState.folders.forEach((f) => expandedFolders.add(f.id));
        }
    });

    function toggleFolder(id: number) {
        if (expandedFolders.has(id)) {
            expandedFolders.delete(id);
        } else {
            expandedFolders.add(id);
        }
        // Force reactivity update
        expandedFolders = new Set(expandedFolders);
    }

    // --- Drag & Drop ---
    function onDragStart(event: DragEvent, feedId: number) {
        if (event.dataTransfer) {
            event.dataTransfer.setData("text/plain", feedId.toString());
            event.dataTransfer.effectAllowed = "move";
        }
    }

    function onDragOver(event: DragEvent) {
        event.preventDefault(); // Necessary to allow dropping
        if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    }

    function onDrop(event: DragEvent, folderId: number) {
        event.preventDefault();
        const feedId = event.dataTransfer?.getData("text/plain");
        if (feedId) {
            appState.moveFeed(parseInt(feedId), folderId);
        }
    }

    // --- Context Menu ---
    function handleContextMenu(event: MouseEvent, type: "folder" | "feed", id: number, name?: string) {
        event.preventDefault();
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
            <div class="folder" ondragover={onDragOver} ondrop={(e) => onDrop(e, folder.id)} role="treeitem" aria-selected="false" aria-expanded={expandedFolders.has(folder.id)} tabindex="0">
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="folder-header" oncontextmenu={(e) => handleContextMenu(e, "folder", folder.id, folder.name)}>
                    <button class="toggle-btn" onclick={() => toggleFolder(folder.id)} aria-label="Toggle {folder.name}">
                        <svg width="10" height="10" viewBox="0 0 10 10" style="transform: rotate({expandedFolders.has(folder.id) ? 90 : 0}deg); transition: transform 0.2s;">
                            <path d="M2,2 L8,5 L2,8" fill="currentColor" />
                        </svg>
                    </button>
                    <span class="folder-name">{folder.name}</span>
                </div>

                {#if expandedFolders.has(folder.id)}
                    <ul class="feed-list" role="group">
                        {#each folder.feeds as feed (feed.id)}
                            <li role="none">
                                <button class="feed-item" class:selected={appState.selectedFeedId === feed.id} onclick={() => appState.selectFeed(feed.id)} oncontextmenu={(e) => handleContextMenu(e, "feed", feed.id)} draggable="true" ondragstart={(e) => onDragStart(e, feed.id)} role="treeitem" aria-selected={appState.selectedFeedId === feed.id}>
                                    <span class="feed-icon">#</span>
                                    {feed.name}
                                </button>
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
        outline: none; /* Focus handled by children or custom styles if needed */
    }

    .folder-header {
        display: flex;
        align-items: center;
        padding: 4px 0;
        cursor: pointer;
        color: var(--text-secondary);
    }

    .folder-header:hover {
        color: var(--text-primary);
    }

    .toggle-btn {
        background: none;
        border: none;
        padding: 4px;
        cursor: pointer;
        color: inherit;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 16px;
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
    }

    .feed-item {
        width: 100%;
        padding: 0.4rem 0.6rem;
        background: transparent;
        border: none;
        text-align: left;
        cursor: pointer;
        border-radius: 6px;
        font-size: 0.9rem;
        color: var(--text-primary);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .feed-icon {
        color: var(--text-secondary);
        font-size: 0.8rem;
        opacity: 0.7;
    }

    .feed-item:hover {
        background-color: var(--bg-hover);
    }

    .feed-item.selected {
        background-color: var(--bg-selected);
        color: var(--text-inverse);
    }

    .feed-item.selected .feed-icon {
        color: rgba(255, 255, 255, 0.7);
    }

    /* Context Menu */
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
