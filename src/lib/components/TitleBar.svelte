<script lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';
import { appState } from '$lib/store.svelte';
import AboutModal from './AboutModal.svelte';

const appWindow = getCurrentWindow();

let showAbout = $state(false);

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
            title="About FeedMee"
        >
            <img src="/feedmee.png" alt="" class="app-icon">
            <span class="app-title">FeedMee</span>
        </span>
    </div>

    <div class="right-section">
        <div class="window-controls">
            <button type="button" class="win-btn" onclick={minimize} aria-label="Minimize">
                <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
                    <path d="M1,5 L9,5" stroke="currentColor" stroke-width="1" />
                </svg>
            </button>
            <button
                type="button"
                class="win-btn"
                onclick={maximize}
                aria-label={isMaximized ? 'Restore' : 'Maximize'}
            >
                {#if isMaximized}
                    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
                        <rect
                            x="3"
                            y="1"
                            width="6"
                            height="6"
                            stroke="currentColor"
                            stroke-width="1"
                            fill="none"
                        />
                        <rect
                            x="1"
                            y="3"
                            width="6"
                            height="6"
                            stroke="currentColor"
                            stroke-width="1"
                            fill="var(--bg-pane)"
                        />
                    </svg>
                {:else}
                    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
                        <rect
                            x="2"
                            y="2"
                            width="6"
                            height="6"
                            stroke="currentColor"
                            stroke-width="1"
                            fill="none"
                        />
                    </svg>
                {/if}
            </button>
            <button type="button" class="win-btn close" onclick={close} aria-label="Close">
                <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
                    <path d="M2,2 L8,8 M8,2 L2,8" stroke="currentColor" stroke-width="1" />
                </svg>
            </button>
        </div>
    </div>
</header>

<AboutModal bind:isOpen={showAbout} onClose={() => (showAbout = false)} />

<style>
.app-brand {
    display: flex;
    align-items: center;
    cursor: pointer;
    border-radius: 4px;
    padding: 2px 4px;
    -webkit-app-region: no-drag;
}

.app-title {
    font-weight: 700;
    font-size: 0.9rem;
    color: var(--text-primary);
    opacity: 0.8;
    margin-right: 4px;
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
.window-controls {
    -webkit-app-region: no-drag;
    z-index: 20;
    position: relative;
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
</style>
