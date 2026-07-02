<script lang="ts">
import { Info, Rss, Settings } from 'lucide-svelte';
import { flip } from 'svelte/animate';
import { appState } from '$lib/store.svelte';
import ContextMenu from './ContextMenu.svelte';
import FeedItem from './nav/FeedItem.svelte';
import FolderGroup from './nav/FolderGroup.svelte';
import NavToolbar from './nav/NavToolbar.svelte';

function openAddDialog() {
    appState.showAddDialog = true;
}

let initialized = false;
let expandTimeout: ReturnType<typeof setTimeout> | null = null;
let expandTargetId: number | null = null;

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
function handleNavDragOver(e: DragEvent) {
    e.preventDefault();

    const el = document.elementFromPoint(e.clientX, e.clientY);
    if (!el) return;

    const folderEl = el.closest('[data-folder-id]') as HTMLElement | null;
    if (folderEl) {
        const folderId = parseInt(folderEl.dataset.folderId ?? '', 10);
        if (Number.isNaN(folderId)) return;

        if (expandTargetId !== folderId) {
            if (expandTimeout) {
                clearTimeout(expandTimeout);
                expandTimeout = null;
            }
            expandTargetId = folderId;
        }

        if (!appState.expandedFolders.has(folderId) && !expandTimeout) {
            expandTimeout = setTimeout(() => {
                const newSet = new Set(appState.expandedFolders);
                newSet.add(folderId);
                appState.expandedFolders = newSet;
                expandTimeout = null;
                expandTargetId = null;
            }, 600);
        }
    } else {
        if (expandTimeout) {
            clearTimeout(expandTimeout);
            expandTimeout = null;
        }
        expandTargetId = null;
    }
}

function onDragLeavePane(e: DragEvent) {
    const nav = e.currentTarget as HTMLElement;
    const related = e.relatedTarget as Node;
    if (!nav.contains(related)) {
        if (expandTimeout) {
            clearTimeout(expandTimeout);
            expandTimeout = null;
        }
        expandTargetId = null;
    }
}

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
    if (cmTarget?.type !== 'folder') return;
    const newName = prompt('Rename Folder:', cmTarget.name);
    if (newName && newName.trim() !== '') {
        appState.renameFolder(cmTarget.id, newName.trim());
    }
    closeContextMenu();
}

function cmRenameFeed() {
    if (cmTarget?.type !== 'feed') return;
    const feed = appState.folders.flatMap((f) => f.feeds).find((f) => f.id === cmTarget?.id);
    appState.editFeedTarget = {
        id: cmTarget?.id,
        name: cmTarget?.name ?? '',
        url: feed?.url ?? '',
    };
    appState.showEditFeedDialog = true;
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

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<nav
    class="pane"
    oncontextmenu={(e) => handleContextMenu(e, 'root', 0)}
    ondragover={handleNavDragOver}
    ondragleave={onDragLeavePane}
>
    <NavToolbar onExpandAll={expandAll} onCollapseAll={collapseAll} />

    <div class="folder-list" onscroll={closeContextMenu}>
        {#each appState.folders.filter((f) => f.id !== 0) as folder (folder.id)}
            <FolderGroup
                {folder}
                isExpanded={appState.expandedFolders.has(folder.id)}
                onToggle={(e) => {
                    e.stopPropagation();
                    toggleFolder(folder.id);
                }}
                onContextMenu={handleContextMenu}
                onFeedsChange={(folderId, feeds) => {
                    const f = appState.folders.find((x) => x.id === folderId);
                    if (f) f.feeds = feeds;
                }}
            />
        {/each}
        {#each appState.folders as folder (folder.id)}
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
                            oncontextmenu={(e) => handleContextMenu(e, 'feed', feed.id, feed.name)}
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
        {/each}
    </div>

    <ContextMenu x={cmX} y={cmY} visible={cmVisible} onClose={closeContextMenu}>
        {#if cmTarget?.type === 'root'}
            <button type="button" onclick={cmCreateFolder}>New Folder</button>
        {:else if cmTarget?.type === 'folder'}
            <button type="button" onclick={cmRename}>Rename Folder</button>
            <button type="button" class="danger" onclick={cmDelete}>Delete Folder</button>
        {:else if cmTarget?.type === 'feed'}
            <button type="button" onclick={cmRenameFeed}>Edit Feed</button>
            <button type="button" class="danger" onclick={cmDelete}>Delete Feed</button>
        {/if}
    </ContextMenu>

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
            title="Manage Content"
            aria-label="Manage Content"
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
