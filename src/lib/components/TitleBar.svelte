<script lang="ts">
    import { appState } from "$lib/store.svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";

    const appWindow = getCurrentWindow();

    function minimize() {
        appWindow.minimize();
    }

    async function maximize() {
        // Explicitly check state for reliable toggling
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

    function handleAddFeed() {
        const url = prompt("Enter RSS Feed URL:");
        if (url && url.trim().length > 0) {
            appState.addFeed(url.trim());
        }
    }
</script>

<header class="titlebar" data-tauri-drag-region>
    <div class="left-section">
        <div class="mac-spacer"></div>
        <span class="app-title">FeedMee</span>
    </div>

    <div class="toolbar">
        <button class="tool-btn" onclick={handleAddFeed} title="Add Feed" aria-label="Add Feed">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19"></line>
                <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
        </button>

        <button class="tool-btn" onclick={() => appState.importOpml()} title="Import OPML" aria-label="Import OPML">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                <polyline points="7 10 12 15 17 10"></polyline>
                <line x1="12" y1="15" x2="12" y2="3"></line>
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

<style>
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

    /* Explicitly disable drag on all interactive elements to ensure clicks work */
    .titlebar button,
    .titlebar input,
    .window-controls,
    .toolbar {
        -webkit-app-region: no-drag;
        z-index: 20; /* Ensure they sit above the drag region */
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

    .win-btn:hover {
        background-color: var(--bg-hover);
    }
    .win-btn.close:hover {
        background-color: #e81123;
        color: white;
    }
</style>
