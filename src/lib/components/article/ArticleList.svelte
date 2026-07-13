<script lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { articleStore, FEED_ID_LATEST, FEED_ID_SAVED, FEED_ID_TODAY, settingsStore } from '$lib/store.svelte';
import type { Article } from '$lib/types';
import { thumbnailCacheKey } from '$lib/utils/thumbnail';
import ArticleCard from './ArticleCard.svelte';

let { onContextMenu, onTagToggle, tagArticleId = null, onScroll: onExternalScroll }: {
    onContextMenu: (e: MouseEvent, article: Article) => void;
    onTagToggle: (e: MouseEvent, article: Article) => void;
    tagArticleId?: number | null;
    onScroll?: () => void;
} = $props();

let listContainer: HTMLElement;

let thumbnailCache = $state<Record<string, string>>({});
let thumbnailPending = new Set<string>();

let thumbnailSize = $derived(settingsStore.settings.thumbnail_size || 0);

async function loadThumbnail(articleUrl: string, imageUrl: string) {
    const size = thumbnailSize;
    const cacheKey = thumbnailCacheKey(imageUrl, articleUrl, size);
    if (cacheKey in thumbnailCache || thumbnailPending.has(cacheKey)) return;
    thumbnailPending.add(cacheKey);
    try {
        const dataUrl = await invoke<string>('get_thumbnail', { url: articleUrl, imageUrl, size });
        thumbnailCache = { ...thumbnailCache, [cacheKey]: dataUrl };
    } catch {
        // leave absent from cache — fallback icon shows
    } finally {
        thumbnailPending.delete(cacheKey);
    }
}

$effect(() => {
    if (!(thumbnailSize > 0)) return;
    for (const article of articleStore.articles) {
        loadThumbnail(article.url, article.image_url);
    }
});

function onScroll() {
    if (!listContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = listContainer;
    if (scrollHeight - scrollTop <= clientHeight + 100) {
        articleStore.loadMore();
    }
    onExternalScroll?.();
}
</script>

<section
    class="pane"
    bind:this={listContainer}
    onscroll={onScroll}
>
    {#if articleStore.articles.length > 0}
        <ul class="article-list">
            {#each articleStore.articles as article (article.id)}
                <li>
                    <ArticleCard
                        {article}
                        isSelected={articleStore.selectedArticle?.id === article.id}
                        isTagOpen={tagArticleId === article.id}
                        {thumbnailSize}
                        {thumbnailCache}
                        {onContextMenu}
                        {onTagToggle}
                    />
                </li>
            {/each}
        </ul>
        {#if articleStore.isLoadingArticles}
            <div class="loading-more">Loading more...</div>
        {/if}
    {:else if articleStore.isLoadingArticles}
        <div class="loading">Loading articles...</div>
    {:else if articleStore.selectedFeedId === FEED_ID_LATEST}
        <div class="empty-state">
            <p>No recent articles.</p>
        </div>
    {:else if articleStore.selectedFeedId === FEED_ID_TODAY}
        <div class="empty-state">
            <p>No articles today.</p>
        </div>
    {:else if articleStore.selectedFeedId === FEED_ID_SAVED}
        <div class="empty-state">
            <p>No saved articles.</p>
        </div>
    {:else if articleStore.selectedFeedId !== null}
        <div class="empty-state">
            <p>No articles in this feed.</p>
        </div>
    {:else if articleStore.selectedFolderId !== null}
        <div class="empty-state">
            <p>No articles in this folder.</p>
        </div>
    {:else}
        <div class="empty-state">
            <p>Select a feed to see articles.</p>
        </div>
    {/if}
</section>

<style>
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
</style>
