<script lang="ts">
    import { appState } from '$lib/store.svelte';
    import type { AppSettings } from '$lib/types';
    import { Keyboard, Settings, X } from 'lucide-svelte';
    import ShortcutsModal from './ShortcutsModal.svelte';

    let settings = $state<AppSettings>({
        feed_refresh_debounce_minutes: 4,
        refresh_all_debounce_minutes: 0,
        auto_update_interval_minutes: 30,
        log_level: 'info',
        default_view_type: 'latest',
        default_view_id: -1,
        auto_collapse_folders: true,
        mark_feed_read_on_exit: false,
    });
    let showShortcuts = $state(false);
    let initialized = $state(false);
    let prevSettings = $state<AppSettings | null>(null);

    $effect(() => {
        const s = appState.settings;
        if (s && 'default_view_type' in s) {
            settings = { ...s };
            if (!initialized) {
                initialized = true;
                prevSettings = { ...settings };
            }
        }
    });

    $effect(() => {
        if (!initialized || !prevSettings) return;
        if (JSON.stringify(settings) === JSON.stringify(prevSettings)) return;
        prevSettings = { ...settings };
        appState.saveSettings(settings, false);
    });

    function cancel() {
        (document.activeElement as HTMLElement)?.blur();
        appState.closeSettings();
    }

    function onKeyDown(e: KeyboardEvent) {
        if (e.key === 'Escape') cancel();
    }
</script>

<svelte:window onkeydown={onKeyDown} />
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-overlay" onclick={cancel} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
        class="modal"
        onclick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        tabindex="-1">
        <div class="modal-header">
            <h3><Settings size={18} /> Settings</h3>
            <button
                class="shortcuts-btn"
                onclick={() => (showShortcuts = true)}
                title="Keyboard Shortcuts">
                <Keyboard size={18} />
            </button>
            <button class="close-btn" onclick={cancel} title="Close">
                <X size={18} />
            </button>
        </div>

        <div class="modal-content">
            <div class="form-container">
                <div class="form-group">
                    <label for="auto-update">Auto Update Interval (min)</label>
                    <input
                        type="number"
                        id="auto-update"
                        bind:value={settings.auto_update_interval_minutes}
                        min="5"
                        title="0 to disable" />
                </div>

                <div class="form-group">
                    <label for="default-view">Default View on Start</label>
                    <select id="default-view" bind:value={settings.default_view_type}>
                        <option value="latest">Latest</option>
                        <option value="saved">Read Later</option>
                        <option value="folder">Folder</option>
                        <option value="feed">Feed</option>
                    </select>
                </div>

                {#if settings.default_view_type === 'folder'}
                    <div class="form-group indent">
                        <label for="default-folder">Folder</label>
                        <select id="default-folder" bind:value={settings.default_view_id}>
                            {#each appState.folders as folder (folder.id)}
                                <option value={folder.id}>{folder.name}</option>
                            {/each}
                        </select>
                    </div>
                {:else if settings.default_view_type === 'feed'}
                    <div class="form-group indent">
                        <label for="default-feed">Feed</label>
                        <select
                            id="default-feed"
                            class="default-feed"
                            bind:value={settings.default_view_id}>
                            {#each appState.folders as folder}
                                {#each folder.feeds as feed (feed.id)}
                                    <option value={feed.id}>{folder.name} / {feed.name}</option>
                                {/each}
                            {/each}
                        </select>
                    </div>
                {/if}

                <div class="form-group">
                    <label for="auto-collapse">Auto Collapse Folders</label>
                    <div class="checkbox-wrap">
                        <input
                            type="checkbox"
                            id="auto-collapse"
                            bind:checked={settings.auto_collapse_folders} />
                    </div>
                </div>

                <div class="form-group">
                    <label for="mark-read-exit">Mark Feed Read on Exit</label>
                    <div class="checkbox-wrap">
                        <input
                            type="checkbox"
                            id="mark-read-exit"
                            bind:checked={settings.mark_feed_read_on_exit} />
                    </div>
                </div>

                <div class="form-group">
                    <label for="log-level">Log Level</label>
                    <select id="log-level" bind:value={settings.log_level}>
                        <option value="error">Error</option>
                        <option value="warn">Warn</option>
                        <option value="info">Info</option>
                        <option value="debug">Debug</option>
                        <option value="trace">Trace</option>
                    </select>
                </div>
            </div>
        </div>
    </div>
</div>

<ShortcutsModal bind:isOpen={showShortcuts} onClose={() => (showShortcuts = false)} />

<style>
    .modal-overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 10000;
        backdrop-filter: blur(2px);
    }

    .modal {
        background: var(--bg-pane);
        border: 1px solid var(--border-color);
        border-radius: 10px;
        width: auto;
        max-width: 500px;
        max-height: 80vh;
        overflow: hidden;
        box-shadow: 0 16px 40px rgba(0, 0, 0, 0.25);
        display: flex;
        flex-direction: column;
    }

    .modal-header {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 1rem 1.25rem;
        border-bottom: 1px solid var(--border-color);
        flex-shrink: 0;
    }

    .modal-header h3 {
        margin: 0;
        flex: 1;
        font-size: 1rem;
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .shortcuts-btn,
    .close-btn {
        background: none;
        border: none;
        color: var(--text-secondary);
        cursor: pointer;
        padding: 4px;
        border-radius: 4px;
        display: flex;
    }

    .shortcuts-btn:hover,
    .close-btn:hover {
        color: var(--text-primary);
        background: var(--bg-hover);
    }

    .modal-content {
        padding: 1rem 1.25rem;
        overflow-y: auto;
    }

    .form-container {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        min-width: 0;
    }

    .form-group {
        display: flex;
        align-items: center;
        gap: 1rem;
        margin-bottom: 1rem;
    }

    .form-group label {
        flex: 0 0 180px;
        font-size: 0.9rem;
        color: var(--text-secondary);
        text-align: right;
    }

    .form-group input,
    .form-group select {
        flex: 1;
        padding: 8px;
        border: 1px solid var(--border-color);
        background: var(--bg-app);
        color: var(--text-primary);
        border-radius: 4px;
        box-sizing: border-box;
        max-width: 200px;
    }

    .form-group.indent {
        padding-left: 20px;
    }

    .form-group.indent label {
        flex: 0 0 160px;
    }

    .checkbox-wrap {
        display: flex;
        align-items: center;
    }

    .checkbox-wrap input[type='checkbox'] {
        width: 16px;
        height: 16px;
        cursor: pointer;
        accent-color: var(--bg-selected);
    }

    select {
        max-height: 300px;
        overflow-y: auto;
    }
</style>
