<script lang="ts">
    import { appState } from '$lib/store.svelte';
    import { getCurrentWindow } from '@tauri-apps/api/window';
    import { Rss, Search, Settings } from 'lucide-svelte';
    import AboutModal from './AboutModal.svelte';

    const appWindow = getCurrentWindow();

    let showAbout = $state(false);
    let showAddDialog = $state(false);
    let newFeedUrl = $state('');
    let selectedFolderId = $state<number | null>(null);

    function minimize() {
        appWindow.minimize();
    }

    let isMaximized = $state(false);

    $effect(() => {
        appWindow.isMaximized().then((v) => (isMaximized = v));
        const unlisten = appWindow.onResized(() => {
            appWindow.isMaximized().then((v) => (isMaximized = v));
        });
        return () => {
            unlisten.then((fn) => fn());
        };
    });

    async function maximize() {
        if (isMaximized) {
            await appWindow.unmaximize();
        } else {
            await appWindow.maximize();
        }
    }

    function close() {
        appWindow.close();
    }

    async function openAddDialog() {
        newFeedUrl = '';
        selectedFolderId = null;
        try {
            const text = await navigator.clipboard.readText();
            if (/^https?:\/\/.+/.test(text.trim())) {
                newFeedUrl = text.trim();
            }
        } catch {
            /* clipboard access denied */
        }
        showAddDialog = true;
    }

    function closeAddDialog() {
        showAddDialog = false;
    }

    function submitAddFeed() {
        if (newFeedUrl.trim().length > 0) {
            appState.addFeed(newFeedUrl.trim(), selectedFolderId);
            closeAddDialog();
        }
    }

    function handleImport() {
        appState.importOpml();
        closeAddDialog();
    }

    function handleExport() {
        appState.exportOpml();
        closeAddDialog();
    }

    function onKeyDown(e: KeyboardEvent) {
        if (e.key === 'Enter') {
            submitAddFeed();
        } else if (e.key === 'Escape') {
            closeAddDialog();
        }
    }

    function focusOnMount(node: HTMLElement) {
        node.focus();
    }

    let searchDebounce: ReturnType<typeof setTimeout> | null = null;

    function onSearchInput(e: Event) {
        const query = (e.target as HTMLInputElement).value;
        if (searchDebounce) clearTimeout(searchDebounce);
        searchDebounce = setTimeout(() => appState.setSearch(query), 250);
    }
</script>

<header class="titlebar" data-tauri-drag-region>
    <div class="left-section">
        <div class="mac-spacer"></div>

        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <span
            class="app-brand"
            onclick={() => (showAbout = true)}
            role="button"
            tabindex="-1"
            title="About FeedMee">
            <img src="/feedmee.png" alt="" class="app-icon" />
            <span class="app-title">FeedMee</span>
        </span>

        <button
            class="tool-btn"
            onclick={() => appState.openSettings()}
            title="Settings"
            aria-label="Settings">
            <Settings size={18} />
        </button>

        <button
            class="tool-btn"
            onclick={openAddDialog}
            title="Add Content"
            aria-label="Add Content">
            <Rss size={18} />
        </button>
    </div>

    <div class="toolbar">
        <div class="search-wrapper">
            <Search class="search-icon" size={18} />
            <input
                type="text"
                placeholder="Search..."
                aria-label="Search articles"
                oninput={onSearchInput}
                value={appState.searchQuery} />
        </div>
    </div>

    <div class="right-section">
        <div class="window-controls">
            <button class="win-btn" onclick={minimize} aria-label="Minimize">
                <svg width="10" height="10" viewBox="0 0 10 10"
                    ><path d="M1,5 L9,5" stroke="currentColor" stroke-width="1" /></svg>
            </button>
            <button
                class="win-btn"
                onclick={maximize}
                aria-label={isMaximized ? 'Restore' : 'Maximize'}>
                {#if isMaximized}
                    <svg width="10" height="10" viewBox="0 0 10 10">
                        <rect
                            x="3"
                            y="1"
                            width="6"
                            height="6"
                            stroke="currentColor"
                            stroke-width="1"
                            fill="none" />
                        <rect
                            x="1"
                            y="3"
                            width="6"
                            height="6"
                            stroke="currentColor"
                            stroke-width="1"
                            fill="var(--bg-pane)" />
                    </svg>
                {:else}
                    <svg width="10" height="10" viewBox="0 0 10 10"
                        ><rect
                            x="2"
                            y="2"
                            width="6"
                            height="6"
                            stroke="currentColor"
                            stroke-width="1"
                            fill="none" /></svg>
                {/if}
            </button>
            <button class="win-btn close" onclick={close} aria-label="Close">
                <svg width="10" height="10" viewBox="0 0 10 10"
                    ><path d="M2,2 L8,8 M8,2 L2,8" stroke="currentColor" stroke-width="1" /></svg>
            </button>
        </div>
    </div>
</header>

<AboutModal bind:isOpen={showAbout} onClose={() => (showAbout = false)} />

{#if showAddDialog}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-overlay" onclick={closeAddDialog}>
        <div class="modal" onclick={(e) => e.stopPropagation()}>
            <h3>Add Content</h3>

            <div class="input-group">
                <input
                    type="text"
                    bind:value={newFeedUrl}
                    placeholder="Enter RSS Feed URL"
                    onkeydown={onKeyDown}
                    use:focusOnMount />
                <button class="primary" onclick={submitAddFeed}>Add Feed</button>
            </div>

            <div class="form-group">
                <label for="folder-select">Add to folder</label>
                <select id="folder-select" bind:value={selectedFolderId}>
                    <option value={null}>Uncategorized</option>
                    {#each appState.folders as folder (folder.id)}
                        <option value={folder.id}>{folder.name}</option>
                    {/each}
                </select>
            </div>

            <div class="divider">
                <span>OR</span>
            </div>

            <button class="secondary" onclick={handleImport}>Import OPML File</button>
            <button class="secondary" onclick={handleExport}>Export OPML File</button>
        </div>
    </div>
{/if}

<style>
    .app-brand {
        display: flex;
        align-items: center;
        cursor: pointer;
        border-radius: 4px;
        padding: 2px 4px;
        -webkit-app-region: no-drag;
    }

    .app-brand:hover .app-title {
        opacity: 1;
    }

    .app-icon {
        width: 20px;
        height: 20px;
        margin-right: 8px;
    }

    .titlebar {
        height: 40px;
        background: var(--bg-pane);
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0;
        user-select: none;
        border-bottom: 1px solid var(--border-color);
        -webkit-app-region: drag;
    }

    .titlebar button,
    .titlebar input,
    .window-controls,
    .toolbar {
        -webkit-app-region: no-drag;
        z-index: 20;
        position: relative;
    }

    .left-section,
    .right-section,
    .toolbar {
        display: flex;
        align-items: center;
        height: 100%;
    }

    .left-section {
        padding-left: 1rem;
        gap: 2px;
    }

    .app-title {
        font-weight: 700;
        font-size: 0.9rem;
        color: var(--text-primary);
        opacity: 0.8;
        margin-right: 4px;
    }

    .toolbar {
        flex: 1;
        justify-content: center;
    }

    .tool-btn {
        width: 30px;
        height: 30px;
        display: flex;
        align-items: center;
        justify-content: center;
        border: none;
        background: transparent;
        color: var(--text-secondary);
        border-radius: 4px;
        cursor: pointer;
        flex-shrink: 0;
    }

    .tool-btn:hover {
        background-color: var(--bg-hover);
        color: var(--text-primary);
    }

    .search-wrapper {
        position: relative;
    }

    :global(.search-icon) {
        color: var(--text-secondary);
    }

    input {
        background: var(--bg-app);
        border: 1px solid var(--border-color);
        color: var(--text-primary);
        padding: 6px 12px;
        border-radius: 4px;
        font-size: 0.85rem;
        width: 280px;
        outline: none;
    }

    input:focus {
        border-color: var(--bg-selected);
    }

    .right-section {
        padding-right: 0;
    }

    .window-controls {
        display: flex;
        height: 100%;
    }

    .win-btn {
        width: 46px;
        height: 100%;
        border: none;
        background: transparent;
        color: var(--text-primary);
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: default;
    }

    .win-btn svg {
        width: 14px;
        height: 14px;
    }

    .win-btn:hover {
        background-color: var(--bg-hover);
    }

    .win-btn.close:hover {
        background-color: #e81123;
        color: white;
    }

    /* Modal Styles */
    .modal-overlay {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 9999;
        backdrop-filter: blur(2px);
    }

    .modal {
        background: var(--bg-app);
        padding: 1.5rem;
        border-radius: 8px;
        width: 400px;
        box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
        border: 1px solid var(--border-color);
    }

    .modal h3 {
        margin: 0 0 1rem 0;
        font-size: 1.1rem;
        color: var(--text-primary);
    }

    .input-group {
        display: flex;
        gap: 8px;
        margin-bottom: 1rem;
    }

    .input-group input {
        flex: 1;
        padding: 8px 12px;
        width: auto;
    }

    button.primary {
        background-color: var(--bg-selected);
        color: white;
        border: none;
        border-radius: 4px;
        padding: 0 12px;
        font-weight: 500;
        cursor: pointer;
    }

    button.primary:hover {
        opacity: 0.9;
    }

    button.secondary {
        width: 100%;
        padding: 8px;
        background: transparent;
        border: 1px solid var(--border-color);
        color: var(--text-primary);
        border-radius: 4px;
        cursor: pointer;
        margin-top: 8px;
    }

    button.secondary:hover {
        background-color: var(--bg-hover);
    }

    .form-group {
        margin-bottom: 1rem;
    }

    .form-group label {
        display: block;
        font-size: 0.8rem;
        color: var(--text-secondary);
        margin-bottom: 4px;
    }

    .form-group select {
        width: 100%;
        padding: 8px;
        background: var(--bg-app);
        border: 1px solid var(--border-color);
        color: var(--text-primary);
        border-radius: 4px;
        font-size: 0.9rem;
        cursor: pointer;
    }

    .divider {
        display: flex;
        align-items: center;
        text-align: center;
        margin: 1rem 0;
        color: var(--text-secondary);
        font-size: 0.8rem;
    }

    .divider::before,
    .divider::after {
        content: '';
        flex: 1;
        border-bottom: 1px solid var(--border-color);
    }

    .divider span {
        padding: 0 10px;
    }
</style>
