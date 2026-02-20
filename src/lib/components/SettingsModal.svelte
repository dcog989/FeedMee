<script lang="ts">
    import { appState } from '$lib/store.svelte';

    let settings = $state({ ...appState.settings });

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
        <h3>Settings</h3>

        <div class="form-group">
            <label for="refresh-rate">Feed Refresh Debounce (Minutes)</label>
            <input
                type="number"
                id="refresh-rate"
                bind:value={settings.feed_refresh_debounce_minutes}
                min="1" />
        </div>

        <div class="form-group">
            <label for="auto-update">Auto Update Interval (Minutes)</label>
            <input
                type="number"
                id="auto-update"
                bind:value={settings.auto_update_interval_minutes}
                min="0" />
            <span class="hint">Set to 0 to disable</span>
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

        <div class="actions">
            <button class="secondary" onclick={cancel}>Cancel</button>
            <button class="primary" onclick={save}>Save</button>
        </div>
    </div>
</div>

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

    h3 {
        margin-top: 0;
        margin-bottom: 1.5rem;
    }

    .form-group {
        margin-bottom: 1rem;
    }

    label {
        display: block;
        font-size: 0.9rem;
        margin-bottom: 0.3rem;
        color: var(--text-secondary);
    }

    input,
    select {
        width: 100%;
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
