<script lang="ts">
    import { tooltip } from "$lib/actions/tooltip.svelte";
    import { appState } from "$lib/store.svelte";

    let { onExpandAll, onCollapseAll } = $props<{
        onExpandAll: () => void;
        onCollapseAll: () => void;
    }>();

    let isRefreshing = $derived(appState.isLoading);

    function createFolder() {
        const name = prompt("New Folder Name:");
        if (name && name.trim()) {
            appState.createFolder(name.trim());
        }
    }
</script>

<div class="nav-toolbar">
    <button onclick={onExpandAll} use:tooltip={"Expand All"} aria-label="Expand All">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"></polyline></svg>
    </button>
    <button onclick={onCollapseAll} use:tooltip={"Collapse All"} aria-label="Collapse All">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="18 15 12 9 6 15"></polyline></svg>
    </button>
    <button onclick={() => appState.refreshAllFeeds()} use:tooltip={"Refresh All"} disabled={isRefreshing} aria-label="Refresh All">
        <svg class:spinning={isRefreshing} width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M23 4v6h-6"></path>
            <path d="M1 20v-6h6"></path>
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
        </svg>
    </button>
    <button onclick={createFolder} use:tooltip={"New Folder"} class="add-folder-btn" aria-label="New Folder">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
    </button>
</div>

<style>
    .nav-toolbar {
        display: flex;
        align-items: center;
        padding: 4px 8px; /* Reduced vertical and horizontal padding */
        gap: 4px;
        border-bottom: 1px solid var(--border-color);
        margin-bottom: 4px;
        height: 32px;
        box-sizing: border-box;
    }

    .nav-toolbar button {
        background: transparent;
        border: none;
        color: var(--text-secondary);
        border-radius: 4px;
        cursor: pointer;
        padding: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 28px;
        height: 28px;
    }

    .nav-toolbar button:hover {
        background-color: var(--bg-hover);
        color: var(--text-primary);
    }

    .nav-toolbar button:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .add-folder-btn {
        margin-left: auto;
    }

    .spinning {
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }
</style>
