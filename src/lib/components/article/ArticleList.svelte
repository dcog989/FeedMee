<script lang="ts">
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { SvelteMap } from "svelte/reactivity";
import { articleStore, FEED_ID_LATEST, FEED_ID_SAVED, FEED_ID_TODAY, settingsStore } from "#lib/store.svelte.js";
import type { Article } from "#lib/types.js";
import { thumbnailCacheKey } from "#lib/utils/thumbnail.js";
import ArticleCard from "./ArticleCard.svelte";

let {
  onContextMenu,
  onTagToggle,
  tagArticleId = null,
  onScroll: onExternalScroll,
}: {
  onContextMenu: (e: MouseEvent, article: Article) => void;
  onTagToggle: (e: MouseEvent, article: Article) => void;
  tagArticleId?: number | null;
  onScroll?: () => void;
} = $props();

let listContainer = $state<HTMLElement>();

// A SvelteMap updates only the cards reading the key it changed, instead of
// copying the whole cache and re-rendering every card per thumbnail.
const thumbnailCache = new SvelteMap<string, string>();
const thumbnailPending = new Set<string>();
const thumbnailFailed = new Set<string>();
const observedArticles = new WeakMap<Element, Article>();
const observedNodes = new Set<HTMLElement>();
let thumbnailObserver: IntersectionObserver | null = null;
let thumbnailLoadsInFlight = 0;
const thumbnailQueue: Array<() => void> = [];

let thumbnailSize = $derived(settingsStore.settings.thumbnail_size || 0);
let listKey = $derived(
  `${articleStore.selectedFeedId ?? articleStore.selectedFolderId}_${articleStore.searchQuery}_${articleStore.sortOrder}`,
);

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
  // No image means the backend would scrape the article page just to look for one.
  if (!article.image_url) return;
  const size = thumbnailSize;
  if (!(size > 0)) return;
  const cacheKey = thumbnailCacheKey(article.image_url, article.url, size);
  if (thumbnailCache.has(cacheKey) || thumbnailPending.has(cacheKey) || thumbnailFailed.has(cacheKey)) return;
  thumbnailPending.add(cacheKey);
  enqueueThumbnail(cacheKey, article.image_url, size);
}

function enqueueThumbnail(cacheKey: string, imageUrl: string, size: number) {
  const epoch = articleStore.thumbnailEpoch;
  const run = async () => {
    thumbnailLoadsInFlight += 1;
    try {
      const path = await invoke<string>("get_thumbnail", { imageUrl, size });
      // The cache may have been cleared while this was in flight.
      if (epoch === articleStore.thumbnailEpoch) thumbnailCache.set(cacheKey, convertFileSrc(path));
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
      observedNodes.add(node);
      ensureThumbnailObserver().observe(node);
    } else {
      observedNodes.delete(node);
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
      observedNodes.delete(node);
      observedArticles.delete(node);
    },
  };
}

$effect(() => {
  void listKey;
  thumbnailFailed.clear();
});

// Clearing the cache deleted the files, so drop the paths and ask again for the
// rows still on screen.
$effect(() => {
  void articleStore.thumbnailEpoch;
  thumbnailCache.clear();
  thumbnailFailed.clear();
  thumbnailPending.clear();
  for (const node of observedNodes) {
    if (!observedArticles.get(node)) continue;
    thumbnailObserver?.unobserve(node);
    thumbnailObserver?.observe(node);
  }
});

const EMPTY_STATE_BY_FEED_ID: Record<number, string> = {
  [FEED_ID_LATEST]: "No recent articles.",
  [FEED_ID_TODAY]: "No articles today.",
  [FEED_ID_SAVED]: "No saved articles.",
};

let emptyStateMessage = $derived.by(() => {
  if (articleStore.searchQuery.trim()) return "No articles match your search.";
  const feedId = articleStore.selectedFeedId;
  if (feedId !== null) {
    return EMPTY_STATE_BY_FEED_ID[feedId] ?? "No articles in this feed.";
  }
  if (articleStore.selectedFolderId !== null) return "No articles in this folder.";
  return "Select a feed to see articles.";
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
  <section class="pane" bind:this={listContainer} onscroll={onScroll}>
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
    {:else}
      <div class="empty-state">
        <p>{emptyStateMessage}</p>
        {#if articleStore.hasMore}
          <button type="button" class="load-more-btn" onclick={() => articleStore.loadMore()}>Load more</button>
        {/if}
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

.load-more-btn {
  margin-top: 0.5rem;
  padding: 6px 12px;
  border: 1px solid var(--border-color);
  background: var(--bg-app);
  color: var(--text-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85rem;
}

.load-more-btn:hover {
  background: var(--bg-hover);
}
</style>
