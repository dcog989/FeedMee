<script lang="ts">
import { Check, Plus, Tags, Trash2, X } from 'lucide-svelte';
import { tooltip } from '$lib/actions/tooltip.svelte';
import { articleStore, tagStore } from '$lib/store.svelte';
import type { Tag } from '$lib/types';

let { articleId, onClose }: { articleId: number; onClose: () => void } = $props();

let allTags = $state<Tag[]>([]);
let activeTagIds = $state<Set<number>>(new Set());
let inputValue = $state('');
let loading = $state(true);

$effect(() => {
  loadAll();
});

async function loadAll() {
  loading = true;
  try {
    const [tags, articleTags] = await Promise.all([tagStore.getAllTags(), tagStore.getArticleTags(articleId)]);
    allTags = tags;
    activeTagIds = new Set(articleTags.map((t) => t.id));
  } catch (e) {
    console.error('Failed to load tags:', e);
  } finally {
    loading = false;
  }
}

function syncHasTags() {
  const has = activeTagIds.size > 0;
  if (articleStore.selectedArticle?.id === articleId) {
    articleStore.selectedArticle = { ...articleStore.selectedArticle, has_tags: has };
  }
  const idx = articleStore.articles.findIndex((a) => a.id === articleId);
  if (idx !== -1 && articleStore.articles[idx].has_tags !== has) {
    const updated = articleStore.articles.slice();
    updated[idx] = { ...updated[idx], has_tags: has };
    articleStore.articles = updated;
  }
}

async function toggleTag(tag: Tag) {
  if (activeTagIds.has(tag.id)) {
    try {
      await tagStore.removeTag(articleId, tag.id);
      activeTagIds = new Set([...activeTagIds].filter((id) => id !== tag.id));
      syncHasTags();
    } catch (e) {
      console.error('Failed to remove tag:', e);
    }
  } else {
    try {
      await tagStore.addTag(articleId, tag.name);
      activeTagIds = new Set([...activeTagIds, tag.id]);
      syncHasTags();
    } catch (e) {
      console.error('Failed to add tag:', e);
    }
  }
}

async function addNewTag() {
  const name = inputValue.trim();
  if (!name) return;
  try {
    const tag = await tagStore.addTag(articleId, name);
    allTags = [...allTags, tag];
    activeTagIds = new Set([...activeTagIds, tag.id]);
    inputValue = '';
    syncHasTags();
  } catch (e) {
    console.error('Failed to add tag:', e);
  }
}

async function deleteTagPermanently(tag: Tag) {
  const confirmed = confirm(`Delete tag "${tag.name}" from all articles?`);
  if (!confirmed) return;
  try {
    await tagStore.deleteTag(tag.id);
    activeTagIds = new Set([...activeTagIds].filter((id) => id !== tag.id));
    allTags = allTags.filter((t) => t.id !== tag.id);
    syncHasTags();
  } catch (e) {
    console.error('Failed to delete tag:', e);
  }
}

function onInputKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    e.preventDefault();
    addNewTag();
  } else if (e.key === 'Escape') {
    onClose();
  }
}
</script>

<div class="tag-manager">
  <div class="tag-header">
    <Tags size={14} />
    <span>Tags</span>
    <button type="button" class="close-btn" onclick={onClose} aria-label="Close">
      <X size={14} />
    </button>
  </div>
  <div class="tag-body">
    {#if loading}
      <span class="loading-text">Loading...</span>
    {:else if allTags.length === 0}
      <span class="empty-text">No tags yet — create one below</span>
    {:else}
      <div class="tag-list">
        {#each allTags as tag (tag.id)}
          <button
            type="button"
            class="tag-row"
            class:active={activeTagIds.has(tag.id)}
            onclick={() => toggleTag(tag)}
            aria-label="{activeTagIds.has(tag.id) ? 'Remove' : 'Add'} tag {tag.name}"
          >
            <span class="tag-check">
              {#if activeTagIds.has(tag.id)}
                <Check size={12} />
              {/if}
            </span>
            <span class="tag-dot" style="background: {tag.color}"></span>
            <span class="tag-name">{tag.name}</span>
            <span
              class="tag-delete"
              role="button"
              tabindex="0"
              aria-label="Delete tag {tag.name}"
              onclick={(e) => { e.stopPropagation(); deleteTagPermanently(tag); }}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); deleteTagPermanently(tag); } }}
              use:tooltip={'Delete tag'}
            >
              <Trash2 size={11} />
            </span>
          </button>
        {/each}
      </div>
    {/if}
    <div class="tag-input-row">
      <input
        type="text"
        placeholder="New tag name..."
        bind:value={inputValue}
        onkeydown={onInputKeydown}
        aria-label="New tag name"
      >
      <button type="button" class="add-btn" onclick={addNewTag} disabled={!inputValue.trim()}>
        <Plus size={14} />
      </button>
    </div>
  </div>
</div>

<style>
.tag-manager {
  background: var(--bg-app);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  min-width: 200px;
  max-width: 260px;
  z-index: 1000;
  overflow: hidden;
}

.tag-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border-color);
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  margin-left: auto;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 2px;
  border-radius: 4px;
  display: flex;
}

.close-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.tag-body {
  padding: 4px 6px;
  max-height: 240px;
  overflow-y: auto;
}

.tag-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
  margin-bottom: 6px;
}

.tag-row {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  background: transparent;
  border: none;
  color: var(--text-primary);
  padding: 5px 6px;
  border-radius: 4px;
  cursor: pointer;
  text-align: left;
  font-size: 0.8rem;
}

.tag-row:hover {
  background: var(--bg-hover);
}

.tag-row.active {
  background: color-mix(in srgb, var(--bg-selected) 12%, transparent);
}

.tag-check {
  width: 16px;
  height: 16px;
  border-radius: 3px;
  border: 1.5px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--bg-selected);
}

.tag-row.active .tag-check {
  background: var(--bg-selected);
  border-color: var(--bg-selected);
  color: white;
}

.tag-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}

.tag-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tag-delete {
  display: flex;
  align-items: center;
  color: var(--text-secondary);
  opacity: 0;
  transition: opacity 0.12s;
  padding: 2px;
  border-radius: 3px;
}

.tag-row:hover .tag-delete {
  opacity: 0.5;
}

.tag-row:hover .tag-delete:hover {
  opacity: 1;
  background: var(--bg-hover);
  color: #e74c3c;
}

.tag-input-row {
  display: flex;
  gap: 4px;
  border-top: 1px solid var(--border-color);
  padding-top: 6px;
}

.tag-input-row input {
  flex: 1;
  background: var(--bg-content);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 0.8rem;
  outline: none;
  min-width: 0;
}

.tag-input-row input:focus {
  border-color: var(--bg-selected);
}

.add-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  padding: 4px;
  border-radius: 4px;
  cursor: pointer;
  width: 28px;
}

.add-btn:hover:not(:disabled) {
  background: var(--bg-hover);
}

.add-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.loading-text,
.empty-text {
  font-size: 0.8rem;
  color: var(--text-secondary);
  display: block;
  margin-bottom: 6px;
  padding: 4px 2px;
}
</style>
