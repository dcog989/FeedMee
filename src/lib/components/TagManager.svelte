<script lang="ts">
import { Tags, X } from 'lucide-svelte';
import { appState } from '$lib/store.svelte';
import type { Tag } from '$lib/types';

let { articleId, onClose }: { articleId: number; onClose: () => void } = $props();

let tags = $state<Tag[]>([]);
let inputValue = $state('');
let loading = $state(true);

$effect(() => {
    loadTags();
});

async function loadTags() {
    loading = true;
    tags = await appState.getArticleTags(articleId);
    loading = false;
}

async function addTag() {
    const name = inputValue.trim();
    if (!name) return;
    try {
        const tag = await appState.addTag(articleId, name);
        tags = [...tags, tag];
        inputValue = '';
    } catch (e) {
        console.error('Failed to add tag:', e);
    }
}

async function removeTag(tagId: number) {
    try {
        await appState.removeTag(articleId, tagId);
        tags = tags.filter((t) => t.id !== tagId);
    } catch (e) {
        console.error('Failed to remove tag:', e);
    }
}

function onInputKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
        e.preventDefault();
        addTag();
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
        {:else if tags.length === 0}
            <span class="empty-text">No tags</span>
        {:else}
            <div class="tag-list">
                {#each tags as tag (tag.id)}
                    <span class="tag-chip" style="--tag-color: {tag.color}">
                        <span class="tag-dot" style="background: {tag.color}"></span>
                        <span class="tag-name">{tag.name}</span>
                        <button
                            type="button"
                            class="tag-remove"
                            onclick={() => removeTag(tag.id)}
                            aria-label="Remove {tag.name}"
                        >
                            <X size={10} />
                        </button>
                    </span>
                {/each}
            </div>
        {/if}
        <div class="tag-input-row">
            <input
                type="text"
                placeholder="Add tag..."
                bind:value={inputValue}
                onkeydown={onInputKeydown}
                aria-label="New tag name"
            >
            <button type="button" class="add-btn" onclick={addTag} disabled={!inputValue.trim()}>
                Add
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
    padding: 8px 10px;
}

.tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-bottom: 8px;
}

.tag-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: color-mix(in srgb, var(--tag-color) 15%, var(--bg-app));
    border: 1px solid color-mix(in srgb, var(--tag-color) 30%, var(--border-color));
    border-radius: 4px;
    padding: 2px 6px;
    font-size: 0.75rem;
}

.tag-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
}

.tag-name {
    color: var(--text-primary);
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.tag-remove {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0;
    display: flex;
    opacity: 0.5;
    transition: opacity 0.15s;
}

.tag-remove:hover {
    opacity: 1;
    color: var(--text-primary);
}

.tag-input-row {
    display: flex;
    gap: 4px;
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
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 0.8rem;
    cursor: pointer;
    white-space: nowrap;
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
    margin-bottom: 8px;
}
</style>
