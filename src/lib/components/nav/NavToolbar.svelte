<script lang="ts">
    import { appState } from "$lib/store.svelte";

    let { onExpandAll, onCollapseAll } = $props<{
        onExpandAll: () => void;
        onCollapseAll: () => void;
    }>();

    function createFolder() {
        const name = prompt("New Folder Name:");
        if (name && name.trim()) {
            appState.createFolder(name.trim());
        }
    }
</script>

<div class="nav-toolbar">
    <button onclick={onExpandAll} title="Expand All">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"></polyline></svg>
    </button>
    <button onclick={onCollapseAll} title="Collapse All">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="18 15 12 9 6 15"></polyline></svg>
    </button>
    <button onclick={createFolder} title="New Folder" class="add-folder-btn">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
    </button>
</div>

<style>
    .nav-toolbar {
        display: flex;
        align-items: center;
        padding: 8px 12px;
        gap: 2px;
        border-bottom: 1px solid var(--border-color);
        margin-bottom: 4px;
    }

    .nav-toolbar button {
        background: transparent;
        border: none;
        color: var(--text-secondary);
        border-radius: 4px;
        cursor: pointer;
        padding: 4px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .nav-toolbar button:hover {
        background-color: var(--bg-hover);
        color: var(--text-primary);
    }

    .add-folder-btn {
        margin-left: auto;
    }
</style>
