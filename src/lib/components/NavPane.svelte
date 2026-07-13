<script lang="ts">
import { Info, Rss, Settings } from 'lucide-svelte';
import { navStore, refreshStore, uiStore } from '$lib/store.svelte';
import { LS_EXPANDED_FOLDERS } from '$lib/utils/persistence';
import FolderGroup from './nav/FolderGroup.svelte';
import NavContextMenu from './nav/NavContextMenu.svelte';
import NavToolbar from './nav/NavToolbar.svelte';
import UncategorizedFeeds from './nav/UncategorizedFeeds.svelte';

function openAddDialog() {
    uiStore.showAddDialog = true;
}

let initialized = false;
let expandTimeout: ReturnType<typeof setTimeout> | null = null;
let expandTargetId: number | null = null;
let ctxMenu: NavContextMenu;
let folderListEl: HTMLDivElement | undefined;

$effect(() => {
    const feedId = navStore.selectedFeedId;
    const folderId = navStore.selectedFolderId;
    const folders = navStore.folders;

    if (folders.length > 0 && (feedId !== null || folderId !== null)) {
        requestAnimationFrame(() => {
            let selector: string | null = null;
            if (feedId !== null && feedId > 0) {
                selector = '.feed-item.selected';
            } else if (folderId !== null) {
                selector = '.folder-header.selected';
            }
            if (selector) {
                folderListEl?.querySelector<HTMLElement>(selector)?.scrollIntoView({ block: 'start' });
            }
        });
    }
});

$effect(() => {
    if (!initialized) {
        const stored = localStorage.getItem(LS_EXPANDED_FOLDERS);
        if (stored) {
            try {
                const ids = JSON.parse(stored);
                navStore.expandedFolders = new Set(ids);
            } catch (e) {
                console.error(e);
            }
        } else {
            const newSet = new Set(navStore.expandedFolders);
            for (const f of navStore.folders) newSet.add(f.id);
            navStore.expandedFolders = newSet;
        }
        initialized = true;
    }
});

$effect(() => {
    if (initialized) {
        localStorage.setItem(
            LS_EXPANDED_FOLDERS,
            JSON.stringify(Array.from(navStore.expandedFolders)),
        );
    }
});

function toggleFolder(id: number) {
    const newSet = new Set(navStore.expandedFolders);
    if (newSet.has(id)) {
        newSet.delete(id);
    } else {
        if (navStore.settings.auto_collapse_folders) {
            newSet.clear();
        }
        newSet.add(id);
    }
    navStore.expandedFolders = newSet;
}

function expandAll() {
    const newSet = new Set<number>();
    for (const f of navStore.folders) newSet.add(f.id);
    navStore.expandedFolders = newSet;
}

function collapseAll() {
    navStore.expandedFolders = new Set();
}

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

        if (!navStore.expandedFolders.has(folderId) && !expandTimeout) {
            expandTimeout = setTimeout(() => {
                const newSet = new Set(navStore.expandedFolders);
                newSet.add(folderId);
                navStore.expandedFolders = newSet;
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
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<nav
    class="pane"
    oncontextmenu={(e) => ctxMenu.show(e, 'root', 0)}
    ondragover={handleNavDragOver}
    ondragleave={onDragLeavePane}
>
    <NavToolbar onExpandAll={expandAll} onCollapseAll={collapseAll} />

    <div class="folder-list" bind:this={folderListEl} onscroll={() => ctxMenu.close()}>
        {#each navStore.folders.filter((f) => f.id !== 0) as folder (folder.id)}
            <FolderGroup
                {folder}
                isExpanded={navStore.expandedFolders.has(folder.id)}
                onToggle={(e) => {
                    e.stopPropagation();
                    toggleFolder(folder.id);
                }}
                onContextMenu={(e, type, id, name) => ctxMenu.show(e, type, id, name)}
                onFeedsChange={(folderId, feeds) => {
                    const f = navStore.folders.find((x) => x.id === folderId);
                    if (f) f.feeds = feeds;
                }}
            />
        {/each}
        {#each navStore.folders as folder (folder.id)}
            <UncategorizedFeeds {folder} onContextMenu={(e, type, id, name) => ctxMenu.show(e, type, id, name)} />
        {/each}
    </div>

    <NavContextMenu bind:this={ctxMenu} />

    <div class="footer-bar">
        <button
            type="button"
            class="footer-btn"
            onclick={() => uiStore.openSettings()}
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
            onclick={() => uiStore.openAbout()}
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
