<script lang="ts">
import { Info, Rss, Settings } from 'lucide-svelte';
import { appState } from '$lib/store.svelte';
import FolderGroup from './nav/FolderGroup.svelte';
import NavToolbar from './nav/NavToolbar.svelte';

function openAddDialog() {
    appState.showAddDialog = true;
}

let initialized = false;
let dragExpandTimeout: ReturnType<typeof setTimeout> | null = null;

// Context Menu State
let cmVisible = $state(false);
let cmX = $state(0);
let cmY = $state(0);
let cmTarget = $state<{
    type: 'folder' | 'feed' | 'root';
    id: number;
    name?: string;
} | null>(null);

// Load/Save Expansion State
$effect(() => {
    if (!initialized) {
        const stored = localStorage.getItem('appState.expandedFolders');
        if (stored) {
            try {
                const ids = JSON.parse(stored);
                appState.expandedFolders = new Set(ids);
            } catch (e) {
                console.error(e);
            }
        } else {
            const newSet = new Set(appState.expandedFolders);
            for (const f of appState.folders) newSet.add(f.id);
            appState.expandedFolders = newSet;
        }
        initialized = true;
    }
});

$effect(() => {
    if (initialized) {
        localStorage.setItem(
            'appState.expandedFolders',
            JSON.stringify(Array.from(appState.expandedFolders)),
        );
    }
});

function toggleFolder(id: number) {
    const newSet = new Set(appState.expandedFolders);
    if (newSet.has(id)) {
        newSet.delete(id);
    } else {
        if (appState.settings.auto_collapse_folders) {
            newSet.clear();
        }
        newSet.add(id);
    }
    appState.expandedFolders = newSet;
}

function expandAll() {
    const newSet = new Set<number>();
    for (const f of appState.folders) newSet.add(f.id);
    appState.expandedFolders = newSet;
}

function collapseAll() {
    appState.expandedFolders = new Set();
}

// --- Drag to Expand Logic ---
function handleExpandHover(folderId: number) {
    if (!appState.expandedFolders.has(folderId)) {
        if (dragExpandTimeout) clearTimeout(dragExpandTimeout);
        dragExpandTimeout = window.setTimeout(() => {
            const newSet = new Set(appState.expandedFolders);
            newSet.add(folderId);
            appState.expandedFolders = newSet;
        }, 600);
    }
}

function onDragLeavePane() {
    if (dragExpandTimeout) {
        clearTimeout(dragExpandTimeout);
        dragExpandTimeout = null;
    }
}

// --- Context Menu ---
function handleContextMenu(
    event: MouseEvent,
    type: 'folder' | 'feed' | 'root',
    id: number,
    name?: string,
) {
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
    if (!cmTarget || cmTarget.type !== 'folder') return;
    const newName = prompt('Rename Folder:', cmTarget.name);
    if (newName && newName.trim() !== '') {
        appState.renameFolder(cmTarget.id, newName.trim());
    }
    closeContextMenu();
}

function cmRenameFeed() {
    if (!cmTarget || cmTarget.type !== 'feed') return;
    const newName = prompt('Rename Feed:', cmTarget.name);
    if (newName && newName.trim() !== '') {
        appState.renameFeed(cmTarget.id, newName.trim());
    }
    closeContextMenu();
}

function cmDelete() {
    if (!cmTarget) return;
    if (cmTarget.type === 'folder') {
        appState.deleteFolder(cmTarget.id);
    } else if (cmTarget.type === 'feed') {
        appState.deleteFeed(cmTarget.id);
    }
    closeContextMenu();
}

function cmCreateFolder() {
    appState.showNewFolderDialog = true;
    closeContextMenu();
}
</script>

<svelte:window onclick={closeContextMenu} />

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<nav
    class="pane"
    oncontextmenu={(e) => handleContextMenu(e, 'root', 0)}
    ondragleave={onDragLeavePane}
>
    <NavToolbar onExpandAll={expandAll} onCollapseAll={collapseAll} />

    <div class="folder-list" role="tree">
        {#each appState.folders as folder (folder.id)}
            <FolderGroup
                {folder}
                isExpanded={appState.expandedFolders.has(folder.id)}
                onToggle={(e) => {
                    e.stopPropagation();
                    toggleFolder(folder.id);
                }}
                onContextMenu={handleContextMenu}
                onExpandHover={handleExpandHover}
                onFeedsChange={(folderId, feeds) => {
                    const f = appState.folders.find((x) => x.id === folderId);
                    if (f) f.feeds = feeds;
                }}
            />
        {/each}
    </div>

    {#if cmVisible}
        <div class="context-menu" style="top: {cmY}px; left: {cmX}px">
            {#if cmTarget?.type === 'root'}
                <button type="button" onclick={cmCreateFolder}>New Folder</button>
            {:else if cmTarget?.type === 'folder'}
                <button type="button" onclick={cmRename}>Rename Folder</button>
                <button type="button" class="danger" onclick={cmDelete}>Delete Folder</button>
            {:else if cmTarget?.type === 'feed'}
                <button type="button" onclick={cmRenameFeed}>Rename Feed</button>
                <button type="button" class="danger" onclick={cmDelete}>Delete Feed</button>
            {/if}
        </div>
    {/if}

    <div class="footer-bar">
        <button
            type="button"
            class="footer-btn"
            onclick={() => appState.openSettings()}
            title="Settings"
            aria-label="Settings"
        >
            <Settings size={18} />
        </button>
        <button
            type="button"
            class="footer-btn"
            onclick={openAddDialog}
            title="Add Content"
            aria-label="Add Content"
        >
            <Rss size={18} />
        </button>
        <span class="footer-spacer"></span>
        <button
            type="button"
            class="footer-btn"
            onclick={() => appState.openAbout()}
            title="About FeedMee"
            aria-label="About FeedMee"
        >
            <Info size={18} />
        </button>
    </div>
</nav>

<style>
.pane {
    background-color: var(--bg-pane);
    border-right: 1px solid var(--border-color);
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    user-select: none;
    padding-top: 4px;
}

.folder-list {
    flex: 1;
    overflow-y: auto;
    padding: 0 1rem 1rem;
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

.footer-bar {
    display: flex;
    align-items: center;
    padding: 4px 8px;
    gap: 4px;
    border-top: 1px solid var(--border-color);
    flex-shrink: 0;
}

.footer-btn {
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    border-radius: 4px;
    cursor: pointer;
    flex-shrink: 0;
}

.footer-btn:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
}

.footer-spacer {
    flex: 1;
}
</style>
