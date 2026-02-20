<script lang="ts">
    import { tooltip } from '$lib/actions/tooltip.svelte';
    import { appState } from '$lib/store.svelte';
    import { ChevronDown, ChevronUp, RefreshCw, FolderPlus } from 'lucide-svelte';

    let { onExpandAll, onCollapseAll } = $props<{
        onExpandAll: () => void;
        onCollapseAll: () => void;
    }>();

    let isRefreshing = $derived(appState.isLoading);
    let refreshAllTooltip = $derived(appState.isAllFresh() ? 'Already fresh!' : 'Refresh All');

    function createFolder() {
        const name = prompt('New Folder Name:');
        if (name && name.trim()) {
            appState.createFolder(name.trim());
        }
    }
</script>

<div class="nav-toolbar">
    <button onclick={onExpandAll} use:tooltip={'Expand All'} aria-label="Expand All">
        <ChevronDown size={18} />
    </button>
    <button onclick={onCollapseAll} use:tooltip={'Collapse All'} aria-label="Collapse All">
        <ChevronUp size={18} />
    </button>
    <button
        onclick={() => appState.refreshAllFeeds()}
        use:tooltip={refreshAllTooltip}
        disabled={isRefreshing}
        aria-label="Refresh All">
        <RefreshCw size={18} class={isRefreshing ? 'spinning' : ''} />
    </button>
    <button
        onclick={createFolder}
        use:tooltip={'New Folder'}
        class="add-folder-btn"
        aria-label="New Folder">
        <FolderPlus size={18} />
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
