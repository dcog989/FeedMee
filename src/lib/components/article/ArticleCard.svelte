<script lang="ts">
import { Bookmark, Image, Tags } from "lucide-svelte";
import { tooltip } from "$lib/actions/tooltip.svelte";
import { articleStore } from "$lib/store.svelte";
import type { Article } from "$lib/types";
import { thumbnailCacheKey } from "$lib/utils/thumbnail";

let {
  article,
  isSelected,
  thumbnailSize,
  thumbnailCache,
  isTagOpen = false,
  onContextMenu,
  onTagToggle,
}: {
  article: Article;
  isSelected: boolean;
  thumbnailSize: number;
  thumbnailCache: Record<string, string>;
  isTagOpen?: boolean;
  onContextMenu: (e: MouseEvent, article: Article) => void;
  onTagToggle: (e: MouseEvent, article: Article) => void;
} = $props();

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Enter" || e.key === " ") {
    e.preventDefault();
    articleStore.selectArticle(article);
  }
}
</script>

<div
  class="article-card"
  class:selected={isSelected}
  class:unread={!article.is_read}
  class:has-thumbnail={thumbnailSize > 0}
  onclick={() => articleStore.selectArticle(article)}
  oncontextmenu={(e) => onContextMenu(e, article)}
  onkeydown={handleKeydown}
  role="button"
  tabindex="0"
>
  {#if thumbnailSize > 0}
    <div class="thumbnail-wrap" style="width:{thumbnailSize}px;height:{thumbnailSize}px">
      {#if thumbnailCache[thumbnailCacheKey(article.image_url, article.url, thumbnailSize)]}
        <img src={thumbnailCache[thumbnailCacheKey(article.image_url, article.url, thumbnailSize)]} alt="">
      {:else}
        <div class="thumb-fallback" style="width:{thumbnailSize}px;height:{thumbnailSize}px">
          <Image size={Math.round(thumbnailSize * 0.4)} />
        </div>
      {/if}
    </div>
  {/if}

  <div class="card-body">
    <span class="title" title={article.url}>{article.title}</span>

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
          class:active={article.has_tags || isTagOpen}
          onclick={(e) => onTagToggle(e, article)}
          use:tooltip={'Tags'}
          aria-label="Tags"
        >
          {#key article.has_tags || isTagOpen}
            <Tags size={14} fill={article.has_tags ? 'currentColor' : 'none'} />
          {/key}
        </button>

        <button
          type="button"
          class="icon-btn"
          class:active={article.is_saved}
          onclick={(e) => {
                    e.stopPropagation();
                    articleStore.toggleSaved(article);
                }}
          use:tooltip={'Read Later'}
          aria-label="Read Later"
        >
          <Bookmark size={14} fill={article.is_saved ? 'currentColor' : 'none'} />
        </button>
      </div>
    </div>
  </div>
</div>

<style>
.article-card {
  display: block;
  width: 100%;
  padding: 0.8rem 1rem;
  text-align: left;
  border: none;
  border-bottom: 1px solid var(--border-color);
  background: transparent;
  cursor: pointer;
  font: inherit;
  color: var(--text-secondary);
  overflow: hidden;
  outline: none;
  box-sizing: border-box;
}

.article-card.has-thumbnail {
  display: flex;
  gap: 0.75rem;
  align-items: center;
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
}

.card-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.title {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  line-clamp: 3;
  overflow: hidden;
  font-family: var(--font-title, var(--font-serif));
  margin-bottom: 0.2rem;
  font-size: 0.95rem;
  font-weight: 300;
  line-height: 1.3;
  text-decoration: line-through;
  opacity: 0.7;
}

.unread .title {
  text-decoration: none;
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
  transition: opacity 0.2s;
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
</style>
