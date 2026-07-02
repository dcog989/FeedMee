<script lang="ts">
import { Search } from 'lucide-svelte';
import { articleStore } from '$lib/store.svelte';

let searchDebounce: ReturnType<typeof setTimeout> | null = null;

function onInput(e: Event) {
    const query = (e.target as HTMLInputElement).value;
    if (searchDebounce) clearTimeout(searchDebounce);
    searchDebounce = setTimeout(() => articleStore.setSearch(query), 250);
}

function onKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
        articleStore.setSearch('');
        (e.target as HTMLInputElement).blur();
    }
}
</script>

<div class="search-wrapper">
    <Search class="search-icon" size={18} />
    <input
        type="text"
        placeholder="Search..."
        aria-label="Search articles"
        oninput={onInput}
        onkeydown={onKeyDown}
        value={articleStore.searchQuery}
    >
</div>

<style>
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
</style>
