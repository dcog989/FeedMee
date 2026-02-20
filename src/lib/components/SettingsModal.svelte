<script lang="ts">
    import { appState } from '$lib/store.svelte';
    import type { AppSettings } from '$lib/types';
    import { Keyboard } from 'lucide-svelte';
    import ShortcutsModal from './ShortcutsModal.svelte';

    interface SettingsWithDefault extends AppSettings {
        default_view_type: string;
        default_view_id: number;
    }

    let settings = $state<SettingsWithDefault>({
        feed_refresh_debounce_minutes: 4,
        refresh_all_debounce_minutes: 0,
        auto_update_interval_minutes: 30,
        log_level: 'info',
        default_view_type: 'latest',
        default_view_id: -1,
    });
    let showShortcuts = $state(false);

    $effect(() => {
        const s = appState.settings as unknown as Record<string, unknown>;
        if (s && 'default_view_type' in s) {
            settings = {
                ...appState.settings,
                default_view_type: (s.default_view_type as string) || 'latest',
                default_view_id: (s.default_view_id as number) ?? -1,
            };
        }
    });

    function save() {
        appState.saveSettings(settings);
    }

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
        aria-label="Settings"
        tabindex="-1">
        <div class="modal-header">
            <h3>Settings</h3>
            <button
                class="shortcuts-btn"
                onclick={() => (showShortcuts = true)}
                title="Keyboard Shortcuts">
                <Keyboard size={18} />
            </button>
        </div>

        <div class="form-group">
            <label for="refresh-rate">Feed Refresh Debounce</label>
            <div class="input-row">
                <input
                    type="number"
                    id="refresh-rate"
                    bind:value={settings.feed_refresh_debounce_minutes}
                    min="1" />
                <span class="unit">min</span>
            </div>
        </div>

        <div class="form-group">
            <label for="auto-update">Auto Update Interval</label>
            <div class="input-row">
                <input
                    type="number"
                    id="auto-update"
                    bind:value={settings.auto_update_interval_minutes}
                    min="0" />
                <span class="unit">min</span>
            </div>
            <span class="hint">Set to 0 to disable</span>
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
                <select id="default-feed" bind:value={settings.default_view_id}>
                    {#each appState.folders as folder}
                        {#each folder.feeds as feed (feed.id)}
                            <option value={feed.id}>{folder.name} / {feed.name}</option>
                        {/each}
                    {/each}
                </select>
            </div>
        {/if}

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

        <div class="actions">
            <button class="secondary" onclick={cancel}>Cancel</button>
            <button class="primary" onclick={save}>Save</button>
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
        padding: 2rem;
        border-radius: 8px;
        width: 400px;
        border: 1px solid var(--border-color);
        color: var(--text-primary);
    }

    .modal-header {
        display: flex;
        align-items: center;
        margin-bottom: 1.5rem;
    }

    h3 {
        margin: 0;
        flex: 1;
    }

    .shortcuts-btn {
        background: none;
        border: none;
        color: var(--text-secondary);
        cursor: pointer;
        padding: 4px;
        border-radius: 4px;
        display: flex;
    }

    .shortcuts-btn:hover {
        color: var(--text-primary);
        background: var(--bg-hover);
    }

    .form-group {
        margin-bottom: 1rem;
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .form-group label {
        flex: 0 0 140px;
        font-size: 0.9rem;
        color: var(--text-secondary);
        margin-bottom: 0;
    }

    .form-group.indent {
        padding-left: 140px;
    }

    .form-group.indent label {
        flex: 0 0 auto;
    }

    .input-row {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .unit {
        font-size: 0.8rem;
        color: var(--text-secondary);
    }

    input,
    select {
        flex: 1;
        padding: 8px;
        border: 1px solid var(--border-color);
        background: var(--bg-app);
        color: var(--text-primary);
        border-radius: 4px;
        box-sizing: border-box;
    }

    .hint {
        font-size: 0.75rem;
        color: var(--text-secondary);
        opacity: 0.8;
    }

    .actions {
        display: flex;
        justify-content: flex-end;
        gap: 10px;
        margin-top: 2rem;
    }

    button {
        padding: 8px 16px;
        border-radius: 4px;
        cursor: pointer;
        border: none;
        font-weight: 500;
    }

    .primary {
        background-color: var(--bg-selected);
        color: white;
    }

    .secondary {
        background-color: transparent;
        border: 1px solid var(--border-color);
        color: var(--text-primary);
    }
</style>
