<script lang="ts">
import { ChevronRight, RefreshCcwDot, RefreshCw, X } from 'lucide-svelte';
import { flip } from 'svelte/animate';
import { tooltip } from '$lib/actions/tooltip.svelte';
import { appState } from '$lib/store.svelte';
import type { Feed, Folder } from '$lib/types';

let { folder, isExpanded, onToggle, onContextMenu, onFeedsChange } = $props<{
    folder: Folder;
    isExpanded: boolean;
    onToggle: (e: MouseEvent) => void;
    onContextMenu: (e: MouseEvent, type: 'folder' | 'feed', id: number, name?: string) => void;
    onFeedsChange: (folderId: number, feeds: Feed[]) => void;
}>();

const FLIP_DURATION = 200;

// --- Native DnD State ---
let dropIndex = $state<number | null>(null);
let unreadCount = $derived(getFolderUnreadCount(folder.feeds));

// --- Native DnD Handlers ---
function handleDragStart(e: DragEvent, feedId: number, feedName: string) {
    const dt = e.dataTransfer;
    if (!dt) return;
    dt.effectAllowed = 'move';
    dt.setData('text/plain', JSON.stringify({ feedId, folderId: folder.id }));

    const root = document.documentElement;
    const style = getComputedStyle(root);
    const bg = style.getPropertyValue('--bg-content').trim() || '#333';
    const text = style.getPropertyValue('--text-primary').trim() || '#fff';
    const pink = style.getPropertyValue('--bg-selected').trim() || '#ec4899';

    const img = document.createElement('div');
    img.textContent = feedName;
    img.style.cssText = `padding:2px 8px;background:${bg};color:${text};border:1px solid ${pink};border-radius:4px;font:8px/1.3 sans-serif;white-space:nowrap;position:absolute;top:-1000px;left:-1000px;pointer-events:none;`;
    document.body.appendChild(img);
    dt.setDragImage(img, 0, 0);
    requestAnimationFrame(() => document.body.removeChild(img));
}

function handleDragOver(e: DragEvent) {
    e.preventDefault();
    if (!isExpanded) return;

    const ul = e.currentTarget as HTMLUListElement;
    const items = Array.from(ul.children as HTMLCollectionOf<HTMLLIElement>);
    const mouseY = e.clientY;

    let idx = items.length;
    for (let i = 0; i < items.length; i++) {
        const rect = items[i].getBoundingClientRect();
        if (mouseY < rect.top + rect.height / 2) {
            idx = i;
            break;
        }
    }
    dropIndex = idx;
}

function handleDragLeaveList(e: DragEvent) {
    const ul = e.currentTarget as HTMLUListElement;
    const related = e.relatedTarget as Node;
    if (!ul.contains(related)) {
        dropIndex = null;
    }
}

function handleDrop(e: DragEvent) {
    e.preventDefault();
    const dt = e.dataTransfer;
    if (!dt) return;
    const data = dt.getData('text/plain');
    if (!data) return;

    const { feedId, folderId: sourceFolderId } = JSON.parse(data);

    const feeds = [...folder.feeds];
    const draggedIndex = feeds.findIndex((f) => f.id === feedId);

    if (draggedIndex !== -1) {
        const [item] = feeds.splice(draggedIndex, 1);
        const idx =
            dropIndex !== null
                ? dropIndex > draggedIndex
                    ? dropIndex - 1
                    : dropIndex
                : feeds.length;
        feeds.splice(idx, 0, item);
        onFeedsChange(folder.id, feeds);
    } else {
        const allFeeds = appState.folders.flatMap((f) => f.feeds);
        const feed = allFeeds.find((f) => f.id === feedId);
        if (!feed) return;
        const idx = dropIndex ?? feeds.length;
        feeds.splice(idx, 0, feed);
        onFeedsChange(folder.id, feeds);
    }

    if (sourceFolderId !== folder.id) {
        appState.moveFeed(feedId, folder.id);
    }

    dropIndex = null;
}

function handleDragEnd() {
    dropIndex = null;
}

function onHeaderDblClick(e: MouseEvent) {
    e.stopPropagation();
    onToggle(e);
}

const FAVICON_TTL = 48 * 60 * 60 * 1000;
const faviconCache = new Map<string, { url: string; time: number }>();

function getFavicon(url: string): string {
    try {
        const domain = new URL(url).hostname;
        const cached = faviconCache.get(domain);
        if (cached && Date.now() - cached.time < FAVICON_TTL) {
            return cached.url;
        }
        const result = `https://icons.duckduckgo.com/ip3/${domain}.ico`;
        faviconCache.set(domain, { url: result, time: Date.now() });
        return result;
    } catch {
        return '';
    }
}

function handleFaviconError(e: Event) {
    const img = e.currentTarget as HTMLImageElement;
    img.style.display = 'none';
    const fallback = img.nextElementSibling;
    if (fallback) fallback.classList.remove('favicon-fallback-hidden');
}

function getFolderUnreadCount(feeds: Feed[]): number {
    return feeds.reduce((acc, feed) => acc + feed.unread_count, 0);
}
</script>

<div
    class="folder"
    role="treeitem"
    aria-expanded={isExpanded}
    aria-selected="false"
    tabindex="-1"
    data-folder-id={folder.id}
>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- biome-ignore lint/a11y/noStaticElementInteractions: oncontextmenu/ondblclick are mouse-only events with no keyboard analog -->
    <div
        class="folder-header"
        class:selected={appState.selectedFolderId === folder.id}
        oncontextmenu={(e) => onContextMenu(e, 'folder', folder.id, folder.name)}
        ondblclick={onHeaderDblClick}
    >
        <button
            type="button"
            class="toggle-icon"
            onclick={onToggle}
            aria-label="Toggle folder"
        >
            <ChevronRight
                size={10}
                style="transform: rotate({isExpanded ? 90 : 0}deg); transition: transform 0.2s;"
            />
        </button>

        <button
            type="button"
            class="folder-name-area"
            onclick={(e) => {
                appState.selectFolder(folder.id);
                onToggle(e);
            }}
        >
            <span class="folder-name">{folder.name}</span>
        </button>

        <button
            type="button"
            class="folder-action-area"
            onclick={(e) => {
                e.stopPropagation();
                appState.requestRefreshFolder(folder.id);
            }}
            aria-label="Refresh folder"
        >
            {#if appState.isFolderUpdating(folder.id)}
                <div class="mini-spinner"></div>
            {:else if unreadCount > 0}
                <span
                    class="badge folder-badge"
                    use:tooltip={appState.isFolderFresh(folder.id)
                        ? 'Already fresh!'
                        : 'Click to refresh folder'}
                    >{unreadCount}</span
                >
            {:else}
                <span
                    class="refresh-icon folder-refresh"
                    use:tooltip={appState.isFolderFresh(folder.id)
                        ? 'Already fresh!'
                        : 'Click to refresh folder'}
                >
                    <RefreshCcwDot size={16} />
                </span>
            {/if}
        </button>
    </div>

    <ul
        class="feed-list"
        class:collapsed={!isExpanded}
        class:drag-over={dropIndex !== null && isExpanded}
        ondragover={handleDragOver}
        ondragleave={handleDragLeaveList}
        ondrop={handleDrop}
    >
        {#if isExpanded}
            {#each folder.feeds as feed, i (feed.id)}
                <li
                    animate:flip={{ duration: FLIP_DURATION }}
                    draggable={true}
                    ondragstart={(e) => handleDragStart(e, feed.id, feed.name)}
                    ondragend={handleDragEnd}
                    class:drop-before={dropIndex === i}
                >
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <div
                        class="feed-item"
                        class:selected={appState.selectedFeedId === feed.id}
                        onclick={(e) => {
                            e.stopPropagation();
                            appState.selectFeed(feed.id);
                        }}
                        oncontextmenu={(e) => onContextMenu(e, 'feed', feed.id, feed.name)}
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
                        <span class="feed-name-wrap">
                            {#if feed.url}
                                <img
                                    src={getFavicon(feed.url)}
                                    alt=""
                                    class="feed-favicon"
                                    loading="lazy"
                                    onerror={handleFaviconError}
                                >
                                <span class="feed-icon favicon-fallback-hidden">#</span>
                            {:else}
                                <span class="feed-icon">#</span>
                            {/if}
                            <span class="feed-name">{feed.name}</span>
                        </span>

                        <!-- Action Area -->
                        <button
                            type="button"
                            class="feed-action-area"
                            onclick={(e) => {
                                e.stopPropagation();
                                appState.requestRefreshFeed(feed.id);
                            }}
                            aria-label="Refresh feed"
                        >
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
                                        : 'Click to refresh'}
                                    >{feed.unread_count}</span
                                >
                            {:else}
                                <span
                                    class="refresh-icon"
                                    use:tooltip={appState.isFeedFresh(feed.id)
                                        ? 'Already fresh!'
                                        : 'Refresh'}
                                >
                                    <RefreshCw size={16} />
                                </span>
                            {/if}
                        </button>
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
    padding: 4px 0.6rem 4px 4px;
    border-left: 3px solid transparent;
    box-sizing: border-box;
    width: 100%;
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
    border: none;
    background: transparent;
    padding: 0;
    font: inherit;
    color: inherit;
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
    border: none;
    background: transparent;
    font: inherit;
    color: inherit;
    text-align: left;
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
    height: 0;
    min-height: 0;
    overflow: hidden;
    padding: 0;
    margin: 0;
    opacity: 0;
}

.feed-list.drag-over {
    outline: 2px solid var(--bg-selected);
    border-radius: 4px;
}

li.drop-before::before {
    content: "";
    position: absolute;
    top: -1px;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--bg-selected);
    z-index: 5;
    pointer-events: none;
}

.feed-list li {
    position: relative;
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

.favicon-fallback-hidden {
    display: none;
}

.feed-action-area {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    min-width: 24px;
    height: 100%;
    cursor: pointer;
    padding-left: 8px;
    border: none;
    background: transparent;
    font: inherit;
    color: inherit;
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

.folder-action-area {
    display: flex;
    align-items: center;
    min-width: 24px;
    margin-left: auto;
    padding-left: 8px;
    justify-content: center;
    cursor: pointer;
    border: none;
    background: transparent;
    font: inherit;
    color: inherit;
}

.folder-refresh {
    opacity: 0.5;
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
    width: 14px;
    height: 14px;
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
