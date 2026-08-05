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

let listContainer = $state<HTMLElement>();

let thumbnailCache = $state<Record<string, string>>({});
const thumbnailPending = new Set<string>();
const thumbnailFailed = new Set<string>();
const observedArticles = new WeakMap<Element, Article>();
let thumbnailObserver: IntersectionObserver | null = null;
let thumbnailLoadsInFlight = 0;
const thumbnailQueue: Array<() => void> = [];

let thumbnailSize = $derived(settingsStore.settings.thumbnail_size || 0);
let listKey = $derived(articleStore.selectedFeedId ?? articleStore.selectedFolderId);

const THUMBNAIL_CONCURRENCY = 8;
const THUMBNAIL_PRELOAD_PX = 250;

function ensureThumbnailObserver(): IntersectionObserver {
    if (!thumbnailObserver) {
        thumbnailObserver = new IntersectionObserver(
            (entries) => {
                for (const entry of entries) {
                    if (!entry.isIntersecting) continue;
                    const article = observedArticles.get(entry.target);
                    if (article) requestThumbnail(article);
                }
            },
            { rootMargin: `${THUMBNAIL_PRELOAD_PX}px` },
        );
    }
    return thumbnailObserver;
}

function requestThumbnail(article: Article) {
    const size = thumbnailSize;
    if (!(size > 0)) return;
    const cacheKey = thumbnailCacheKey(article.image_url, article.url, size);
    if (cacheKey in thumbnailCache || thumbnailPending.has(cacheKey) || thumbnailFailed.has(cacheKey)) return;
    thumbnailPending.add(cacheKey);
    enqueueThumbnail(cacheKey, article.url, article.image_url, size);
}

function enqueueThumbnail(cacheKey: string, articleUrl: string, imageUrl: string, size: number) {
    const run = async () => {
        thumbnailLoadsInFlight += 1;
        try {
            const dataUrl = await invoke<string>('get_thumbnail', { url: articleUrl, imageUrl, size });
            thumbnailCache = { ...thumbnailCache, [cacheKey]: dataUrl };
        } catch {
            thumbnailFailed.add(cacheKey);
        } finally {
            thumbnailLoadsInFlight -= 1;
            thumbnailPending.delete(cacheKey);
            const next = thumbnailQueue.shift();
            if (next) next();
        }
    };
    if (thumbnailLoadsInFlight < THUMBNAIL_CONCURRENCY) {
        run();
    } else {
        thumbnailQueue.push(run);
    }
}

function observeThumbnails(node: HTMLElement, opts: { article: Article; thumbnailSize: number }) {
    const setup = (article: Article, size: number) => {
        observedArticles.set(node, article);
        if (size > 0) {
            ensureThumbnailObserver().observe(node);
        } else {
            thumbnailObserver?.unobserve(node);
        }
    };
    setup(opts.article, opts.thumbnailSize);
    return {
        update(next: { article: Article; thumbnailSize: number }) {
            setup(next.article, next.thumbnailSize);
        },
        destroy() {
            thumbnailObserver?.unobserve(node);
            observedArticles.delete(node);
        },
    };
}

$effect(() => {
    void listKey;
    thumbnailFailed.clear();
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

{#key listKey}
    <section
        class="pane"
        bind:this={listContainer}
        onscroll={onScroll}
    >
        {#if articleStore.articles.length > 0}
            <ul class="article-list">
                {#each articleStore.articles as article (article.id)}
                    <li use:observeThumbnails={{ article, thumbnailSize }}>
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
{/key}

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
