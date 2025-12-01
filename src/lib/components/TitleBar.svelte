<script lang="ts">
    import { appState } from "$lib/store.svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";

    const appWindow = getCurrentWindow();

    let showAddDialog = $state(false);
    let newFeedUrl = $state("");

    function minimize() {
        appWindow.minimize();
    }

    async function maximize() {
        const isMaximized = await appWindow.isMaximized();
        if (isMaximized) {
            await appWindow.unmaximize();
        } else {
            await appWindow.maximize();
        }
    }

    function close() {
        appWindow.close();
    }

    function openAddDialog() {
        newFeedUrl = "";
        showAddDialog = true;
    }

    function closeAddDialog() {
        showAddDialog = false;
    }

    function submitAddFeed() {
        if (newFeedUrl && newFeedUrl.trim().length > 0) {
            appState.addFeed(newFeedUrl.trim());
            closeAddDialog();
        }
    }

    function handleImport() {
        appState.importOpml();
        closeAddDialog();
    }

    function onKeyDown(e: KeyboardEvent) {
        if (e.key === "Enter") {
            submitAddFeed();
        } else if (e.key === "Escape") {
            closeAddDialog();
        }
    }

    function focusOnMount(node: HTMLElement) {
        node.focus();
    }
</script>

<header class="titlebar" data-tauri-drag-region>
    <div class="left-section">
        <div class="mac-spacer"></div>
        <img src="/feedmee.png" alt="" class="app-icon" />
        <span class="app-title">FeedMee</span>
    </div>

    <div class="toolbar">
        <button class="tool-btn" onclick={openAddDialog} title="Add Content" aria-label="Add Content">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19"></line>
                <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
        </button>

        <button class="tool-btn" onclick={() => appState.exportOpml()} title="Export OPML" aria-label="Export OPML">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"></path>
                <polyline points="16 6 12 2 8 6"></polyline>
                <line x1="12" y1="2" x2="12" y2="15"></line>
            </svg>
        </button>

        <div class="search-wrapper">
            <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8"></circle>
                <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
            </svg>
            <input type="text" placeholder="Search..." aria-label="Search articles" />
        </div>
    </div>

    <div class="right-section">
        <button class="tool-btn" title="Settings" aria-label="Settings">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="3" y1="12" x2="21" y2="12"></line>
                <line x1="3" y1="6" x2="21" y2="6"></line>
                <line x1="3" y1="18" x2="21" y2="18"></line>
            </svg>
        </button>

        <div class="window-controls">
            <button class="win-btn" onclick={minimize} aria-label="Minimize">
                <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1,5 L9,5" stroke="currentColor" stroke-width="1" /></svg>
            </button>
            <button class="win-btn" onclick={maximize} aria-label="Maximize">
                <svg width="10" height="10" viewBox="0 0 10 10"><rect x="2" y="2" width="6" height="6" stroke="currentColor" stroke-width="1" fill="none" /></svg>
            </button>
            <button class="win-btn close" onclick={close} aria-label="Close">
                <svg width="10" height="10" viewBox="0 0 10 10"><path d="M2,2 L8,8 M8,2 L2,8" stroke="currentColor" stroke-width="1" /></svg>
            </button>
        </div>
    </div>
</header>

{#if showAddDialog}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-overlay" onclick={closeAddDialog}>
        <div class="modal" onclick={(e) => e.stopPropagation()}>
            <h3>Add Content</h3>
            <div class="input-group">
                <input type="text" bind:value={newFeedUrl} placeholder="Enter RSS Feed URL" onkeydown={onKeyDown} use:focusOnMount />
                <button class="primary" onclick={submitAddFeed}>Add Feed</button>
            </div>

            <div class="divider">
                <span>OR</span>
            </div>

            <button class="secondary" onclick={handleImport}>Import OPML File</button>
        </div>
    </div>
{/if}

<style>
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
        min-width: 200px;
    }

    .app-title {
        font-weight: 700;
        font-size: 0.9rem;
        color: var(--text-primary);
        opacity: 0.8;
    }

    .toolbar {
        gap: 0.5rem;
        flex: 1;
        justify-content: center;
    }

    .tool-btn {
        width: 32px;
        height: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
        border: none;
        background: transparent;
        color: var(--text-secondary);
        border-radius: 4px;
        cursor: pointer;
    }

    .tool-btn:hover {
        background-color: var(--bg-hover);
        color: var(--text-primary);
    }

    .search-wrapper {
        position: relative;
        margin-left: 1rem;
    }

    .search-icon {
        position: absolute;
        left: 8px;
        top: 50%;
        transform: translateY(-50%);
        color: var(--text-secondary);
        pointer-events: none;
    }

    input {
        background: var(--bg-app);
        border: 1px solid var(--border-color);
        color: var(--text-primary);
        padding: 4px 8px 4px 28px;
        border-radius: 4px;
        font-size: 0.85rem;
        width: 200px;
        outline: none;
    }

    input:focus {
        border-color: var(--bg-selected);
    }

    .right-section {
        padding-right: 0;
        gap: 0.5rem;
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
        top: 0;
        left: 0;
        width: 100vw;
        height: 100vh;
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
    }

    button.secondary:hover {
        background-color: var(--bg-hover);
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
        content: "";
        flex: 1;
        border-bottom: 1px solid var(--border-color);
    }

    .divider span {
        padding: 0 10px;
    }
</style>
