<script lang="ts">
import { Bookmark, FileText, Tags } from "lucide-svelte";
import { tooltip, tooltipState } from "$lib/actions/tooltip.svelte";
import TagPopover from "$lib/components/TagPopover.svelte";
import { articleStore } from "$lib/store.svelte";
import type { Article } from "$lib/types";

let {
  article,
  isLoadingFull = false,
  hasFullContent = false,
  onLoadFullContent,
}: {
  article: Article;
  isLoadingFull?: boolean;
  hasFullContent?: boolean;
  onLoadFullContent?: () => void;
} = $props();

let tagArticleId = $state<number | null>(null);
let tagX = $state(0);
let tagY = $state(0);
let isSaved = $derived(article.is_saved);
</script>

<div class="meta-actions">
  <button
    type="button"
    class="action-btn"
    class:active={article.has_tags || tagArticleId !== null}
    onclick={(e) => {
            tooltipState.visible = false;
            const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
            tagX = rect.left;
            tagY = rect.bottom + 4;
            tagArticleId = tagArticleId === article.id ? null : article.id;
        }}
    use:tooltip={'Tags'}
    aria-label="Tags"
  >
    {#key article.has_tags || tagArticleId !== null}
      <Tags size={18} fill={article.has_tags ? 'currentColor' : 'none'} />
    {/key}
  </button>

  <button
    type="button"
    class="action-btn"
    class:active={isSaved}
    onclick={() => articleStore.toggleSaved(article)}
    use:tooltip={'Read Later'}
    aria-label="Read Later"
  >
    <Bookmark size={18} fill={isSaved ? 'currentColor' : 'none'} />
  </button>

  <button
    type="button"
    class="action-btn"
    onclick={onLoadFullContent}
    use:tooltip={'Load Full Content'}
    disabled={isLoadingFull || hasFullContent}
    aria-label="Load Full Content"
  >
    {#if isLoadingFull}
      <span class="spinner"></span>
    {:else}
      <FileText size={18} />
    {/if}
  </button>
</div>

<TagPopover articleId={tagArticleId} x={tagX} y={tagY} onClose={() => { tagArticleId = null; }} />

<style>
.meta-actions {
  display: flex;
  gap: 0.5rem;
}

.action-btn {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  padding: 6px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition:
    background-color 0.2s,
    opacity 0.2s;
}

.action-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.action-btn.active {
  color: var(--bg-selected);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--text-secondary);
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}
</style>
