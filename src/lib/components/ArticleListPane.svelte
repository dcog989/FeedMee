<script lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { ArrowUpDown, Bookmark, CheckCheck, Clock, Image, Search, Tags } from 'lucide-svelte';
import { tooltip } from '$lib/actions/tooltip.svelte';
import { appState, FEED_ID_LATEST, FEED_ID_SAVED } from '$lib/store.svelte';
import type { Article } from '$lib/types';
import TagManager from './TagManager.svelte';

let listContainer: HTMLElement;

let searchDebounce: ReturnType<typeof setTimeout> | null = null;

let thumbnailCache = $state<Record<string, string>>({});
let thumbnailPending = new Set<string>();

async function loadThumbnail(articleUrl: string, imageUrl: string) {
    const cacheKey = imageUrl || articleUrl;
    if (cacheKey in thumbnailCache || thumbnailPending.has(cacheKey)) return;
    thumbnailPending.add(cacheKey);
    try {
        const dataUrl = await invoke<string>('get_thumbnail', { url: articleUrl, imageUrl });
        thumbnailCache[cacheKey] = dataUrl;
    } catch {
        // leave absent from cache — fallback icon shows
    } finally {
        thumbnailPending.delete(cacheKey);
    }
}

$effect(() => {
    if (!appState.settings.show_thumbnails) return;
    for (const article of appState.articles) {
        loadThumbnail(article.url, article.image_url);
    }
});

function onSearchInput(e: Event) {
    const query = (e.target as HTMLInputElement).value;
    if (searchDebounce) clearTimeout(searchDebounce);
    searchDebounce = setTimeout(() => appState.setSearch(query), 250);
}

function onSearchKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
        appState.setSearch('');
        (e.target as HTMLInputElement).blur();
    }
}

function onScroll() {
    if (!listContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = listContainer;
    if (scrollHeight - scrollTop <= clientHeight + 100) {
        appState.loadMore();
    }
}

function handleKeydown(e: KeyboardEvent, article: Article) {
    if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        appState.selectArticle(article);
    }
}

// --- Tag Manager ---
let tagArticleId = $state<number | null>(null);
let tagX = $state(0);
let tagY = $state(0);

function toggleTagManager(e: MouseEvent, article: Article) {
    e.stopPropagation();
    if (tagArticleId === article.id) {
        tagArticleId = null;
    } else {
        tagArticleId = article.id;
        const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
        tagX = rect.left;
        tagY = rect.bottom + 4;
    }
}

// --- Article Context Menu ---
let cmVisible = $state(false);
let cmX = $state(0);
let cmY = $state(0);
let cmArticle = $state<Article | null>(null);

function openContextMenu(e: MouseEvent, article: Article) {
    e.preventDefault();
    e.stopPropagation();
    cmArticle = article;
    cmX = e.clientX;
    cmY = e.clientY;
    cmVisible = true;
}

function closeContextMenu() {
    cmVisible = false;
    cmArticle = null;
}

function closeTagManager() {
    tagArticleId = null;
}

function cmOpenInBrowser() {
    if (cmArticle?.url) window.open(cmArticle.url, '_blank');
    closeContextMenu();
}

function cmToggleRead() {
    if (!cmArticle) return;
    const newRead = !cmArticle.is_read;
    cmArticle.is_read = newRead;
    invoke('mark_article_read', { id: cmArticle.id, read: newRead }).catch(() => {
        if (cmArticle) cmArticle.is_read = !newRead;
    });
    closeContextMenu();
}

function cmToggleSaved() {
    if (!cmArticle) return;
    appState.toggleSaved(cmArticle);
    closeContextMenu();
}
</script>

<svelte:window
    onclick={closeContextMenu}
    onkeydown={(e) => { if (e.key === 'Escape') { closeContextMenu(); closeTagManager(); } }}
/>

<div class="pane-wrapper">
    <div class="search-wrapper">
        <Search class="search-icon" size={18} />
        <input
            type="text"
            placeholder="Search..."
            aria-label="Search articles"
            oninput={onSearchInput}
            onkeydown={onSearchKeyDown}
            value={appState.searchQuery}
        >
    </div>

    <div class="list-toolbar">
        <div class="toolbar-left">
            <button
                type="button"
                class="tool-btn"
                class:active={appState.selectedFeedId === FEED_ID_LATEST}
                onclick={() => appState.selectFeed(FEED_ID_LATEST)}
                use:tooltip={'Latest'}
                aria-label="Latest"
            >
                <Clock size={18} />
            </button>
            <button
                type="button"
                class="tool-btn"
                class:active={appState.selectedFeedId === FEED_ID_SAVED}
                onclick={() => appState.selectFeed(FEED_ID_SAVED)}
                use:tooltip={'Read Later'}
                aria-label="Read Later"
            >
                <Bookmark size={18} />
            </button>
        </div>
        <div class="toolbar-right">
            <button
                type="button"
                class="tool-btn"
                onclick={() => appState.setSortOrder(appState.sortOrder === 'desc' ? 'asc' : 'desc')}
                use:tooltip={appState.sortOrder === 'desc'
                    ? 'Sort: Newest First'
                    : 'Sort: Oldest First'}
                aria-label={appState.sortOrder === 'desc' ? 'Sort Newest First' : 'Sort Oldest First'}
            >
                <ArrowUpDown size={20} />
            </button>

            <button
                type="button"
                class="tool-btn"
                onclick={() => appState.markAllRead()}
                use:tooltip={'Mark All Read'}
                aria-label="Mark All Read"
            >
                <CheckCheck size={20} />
            </button>
        </div>
    </div>

    <section
        class="pane"
        bind:this={listContainer}
        onscroll={() => { onScroll(); closeContextMenu(); }}
    >
        {#if appState.articles.length > 0}
            <ul class="article-list">
                {#each appState.articles as article (article.id)}
                    <li>
                        <div
                            class="article-card"
                            class:selected={appState.selectedArticle?.id === article.id}
                            class:unread={!article.is_read}
                            class:has-thumbnail={appState.settings.show_thumbnails}
                            onclick={() => appState.selectArticle(article)}
                            oncontextmenu={(e) => openContextMenu(e, article)}
                            onkeydown={(e) => handleKeydown(e, article)}
                            role="button"
                            tabindex="0"
                        >
                            {#if appState.settings.show_thumbnails}
                                <div class="thumbnail-wrap">
                                    {#if thumbnailCache[article.image_url || article.url]}
                                        <img
                                            src={thumbnailCache[article.image_url || article.url]}
                                            alt=""
                                        >
                                    {:else}
                                        <div class="thumb-fallback">
                                            <Image size={22} />
                                        </div>
                                    {/if}
                                </div>
                            {/if}

                            <div class="card-body">
                                <span class="title" title={article.title}>{article.title}</span>

                                <div class="meta-line">
                                    <div class="meta-left">
                                        <span class="date"
                                            >{new Date(
                                                article.timestamp * 1000,
                                            ).toLocaleDateString()}</span
                                        >
                                        <span class="separator">•</span>
                                        <span class="author">{article.author}</span>
                                    </div>

                                    <div class="actions">
                                        <button
                                            type="button"
                                            class="icon-btn"
                                            class:active={article.has_tags || tagArticleId === article.id}
                                            onclick={(e) => toggleTagManager(e, article)}
                                            use:tooltip={'Tags'}
                                            aria-label="Tags"
                                        >
                                            <Tags
                                                size={14}
                                                fill={article.has_tags ? 'currentColor' : 'none'}
                                            />
                                        </button>

                                        <button
                                            type="button"
                                            class="icon-btn"
                                            class:active={article.is_saved}
                                            onclick={(e) => {
                                            e.stopPropagation();
                                            appState.toggleSaved(article);
                                        }}
                                            use:tooltip={'Read Later'}
                                            aria-label="Read Later"
                                        >
                                            <Bookmark
                                                size={14}
                                                fill={article.is_saved ? 'currentColor' : 'none'}
                                            />
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </li>
                {/each}
            </ul>
            {#if appState.isLoadingArticles}
                <div class="loading-more">Loading more...</div>
            {/if}
        {:else if appState.isLoadingArticles}
            <div class="loading">Loading articles...</div>
        {:else if appState.selectedFeedId === FEED_ID_LATEST}
            <div class="empty-state">
                <p>No recent articles.</p>
            </div>
        {:else if appState.selectedFeedId === FEED_ID_SAVED}
            <div class="empty-state">
                <p>No saved articles.</p>
            </div>
        {:else if appState.selectedFeedId !== null}
            <div class="empty-state">
                <p>No articles in this feed.</p>
            </div>
        {:else if appState.selectedFolderId !== null}
            <div class="empty-state">
                <p>No articles in this folder.</p>
            </div>
        {:else}
            <div class="empty-state">
                <p>Select a feed to see articles.</p>
            </div>
        {/if}
    </section>
</div>

{#if tagArticleId !== null}
    <div class="tag-backdrop" onclick={() => { tagArticleId = null; }} role="presentation"></div>
    <div class="tag-popover" style="top: {tagY}px; left: {tagX}px">
        <TagManager articleId={tagArticleId} onClose={() => { tagArticleId = null; }} />
    </div>
{/if}

{#if cmVisible}
    <div class="context-menu" style="top: {cmY}px; left: {cmX}px" role="menu">
        <button type="button" onclick={cmOpenInBrowser}>Open in Browser</button>
        <button type="button" onclick={cmToggleRead}>
            {cmArticle?.is_read ? 'Mark Unread' : 'Mark Read'}
        </button>
        <button type="button" onclick={cmToggleSaved}>
            {cmArticle?.is_saved ? 'Remove Bookmark' : 'Bookmark'}
        </button>
    </div>
{/if}

<style>
.pane-wrapper {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--bg-article, var(--bg-content));
    border-right: 1px solid var(--border-color);
}

.list-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-pane);
    flex-shrink: 0;
    height: 32px;
    box-sizing: border-box;
}

.toolbar-left,
.toolbar-right {
    display: flex;
    align-items: center;
    gap: 4px;
}

.tool-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    border-radius: 4px;
    width: 32px;
    height: 32px;
}

.tool-btn:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
}

.tool-btn.active {
    color: var(--bg-selected);
}

.pane {
    flex: 1;
    overflow-y: auto;
    height: 100%;
    box-sizing: border-box;
}

.article-list {
    list-style: none;
    margin: 0;
    padding: 0;
}

.loading,
.empty-state,
.loading-more {
    padding: 2rem 1rem;
    text-align: center;
    color: var(--text-secondary);
    font-size: 0.9rem;
}

.article-card {
    display: block;
    width: 100%;
    padding: 0.8rem 1rem;
    text-align: left;
    border-bottom: 1px solid var(--border-color);
    background: transparent;
    cursor: pointer;
    color: var(--text-secondary);
    overflow: hidden;
    outline: none;
    box-sizing: border-box;
}

.article-card.has-thumbnail {
    display: flex;
    gap: 0.75rem;
    align-items: flex-start;
}

.article-card.unread {
    color: var(--text-primary);
    font-weight: 400;
}

.article-card:hover {
    background-color: var(--bg-hover);
}

.article-card:focus-visible {
    background-color: var(--bg-hover);
    box-shadow: inset 4px 0 0 var(--border-color);
}

.article-card.selected {
    background-color: var(--bg-hover);
    border-left: 4px solid var(--bg-selected);
    padding-left: calc(1rem - 4px);
    color: var(--text-primary);
}

.thumbnail-wrap {
    width: 56px;
    height: 56px;
    flex-shrink: 0;
    border-radius: 6px;
    overflow: hidden;
    background: var(--bg-hover);
    display: flex;
    align-items: center;
    justify-content: center;
}

.thumbnail-wrap img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
}

.thumb-fallback {
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
}

.card-body {
    flex: 1;
    min-width: 0;
}

.title {
    display: block;
    font-family: var(--font-title, var(--font-serif));
    margin-bottom: 0.3rem;
    font-size: 0.95rem;
    font-weight: 300;
    line-height: 1.3;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    /* Strikethrough for read articles (which are NOT .unread) */
    text-decoration: line-through;
    opacity: 0.7;
}

.unread .title {
    text-decoration: none; /* Reset for unread */
    opacity: 1;
}

.meta-line {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--text-secondary);
    margin-top: 0.4rem;
    font-weight: 400;
}

.meta-left {
    display: flex;
    gap: 6px;
    align-items: center;
    overflow: hidden;
}

.separator {
    opacity: 0.5;
}

.author {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.actions {
    display: flex;
    gap: 8px;
}

.icon-btn {
    display: flex;
    align-items: center;
    color: var(--text-secondary);
    opacity: 0.4;
    transition: all 0.2s;
    cursor: pointer;
    background: transparent;
    border: none;
    padding: 0;
}

.icon-btn:hover {
    opacity: 1;
    color: var(--text-primary);
}

.icon-btn.active {
    color: var(--bg-selected);
    opacity: 1;
}

.tag-backdrop {
    position: fixed;
    inset: 0;
    z-index: 999;
}

.tag-popover {
    position: fixed;
    z-index: 1000;
}

.search-wrapper {
    position: relative;
    padding: 4px 8px;
    flex-shrink: 0;
    background: var(--bg-pane);
    border-bottom: 1px solid var(--border-color);
}

.search-wrapper input {
    background: var(--bg-app);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    padding: 6px 12px 6px 32px;
    border-radius: 4px;
    font-size: 0.85rem;
    width: 100%;
    outline: none;
    box-sizing: border-box;
}

.search-wrapper input:focus {
    border-color: var(--bg-selected);
}

.search-wrapper :global(.search-icon) {
    position: absolute;
    left: 14px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-secondary);
    pointer-events: none;
}

.context-menu {
    position: fixed;
    background: var(--bg-app);
    border: 1px solid var(--border-color);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    border-radius: 6px;
    padding: 4px;
    z-index: 1000;
    min-width: 150px;
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
</style>
