<script lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { Keyboard, Settings, X } from 'lucide-svelte';
import { appState } from '$lib/store.svelte';
import type { AppSettings } from '$lib/types';
import ShortcutsModal from './ShortcutsModal.svelte';

let settings = $state<AppSettings>({
    feed_refresh_debounce_minutes: 4,
    auto_update_interval_minutes: 30,
    log_level: 'info',
    default_view_type: 'latest',
    default_view_id: -1,
    auto_collapse_folders: true,
    mark_feed_read_on_exit: false,
    article_title_font: '',
    article_body_font: '',
    article_title_color: '',
    article_body_color: '',
    article_bg_color: '',
    thumbnail_size: 0,
    article_retention_days: 90,
});
let showShortcuts = $state(false);
let initialized = $state(false);
let prevSettings = $state<AppSettings | null>(null);
let saveTimer: ReturnType<typeof setTimeout> | null = null;

async function pickFont(target: 'title' | 'body') {
    try {
        const font = await invoke<string>('pick_system_font');
        if (target === 'title') settings.article_title_font = font;
        else settings.article_body_font = font;
    } catch (e) {
        const msg = String(e);
        if (msg !== 'Font selection cancelled') {
            appState.alert(msg);
        }
    }
}

$effect(() => {
    const s = appState.settings;
    if (s && 'default_view_type' in s) {
        const cs = getComputedStyle(document.documentElement);
        settings = {
            ...s,
            article_title_color:
                s.article_title_color || cs.getPropertyValue('--accent-muted').trim(),
            article_body_color:
                s.article_body_color || cs.getPropertyValue('--text-primary').trim(),
            article_bg_color: s.article_bg_color || cs.getPropertyValue('--bg-reading').trim(),
        };
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
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => appState.saveSettings(settings, false), 500);
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
        tabindex="-1"
    >
        <div class="modal-header">
            <h3><Settings size={18} /> Settings</h3>
            <button
                type="button"
                class="shortcuts-btn"
                onclick={() => (showShortcuts = true)}
                title="Keyboard Shortcuts"
            >
                <Keyboard size={18} />
            </button>
            <button type="button" class="close-btn" onclick={cancel} title="Close">
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
                        title="0 to disable"
                    >
                </div>

                <div class="form-group">
                    <label for="default-view">View on Startup</label>
                    <select id="default-view" bind:value={settings.default_view_type}>
                        <option value="latest">Latest</option>
                        <option value="saved">Read Later</option>
                        <option value="last">Last Folder or Feed</option>
                        <option value="folder">Folder…</option>
                        <option value="feed">Feed…</option>
                    </select>
                </div>

                {#if settings.default_view_type === 'folder'}
                    <div class="form-group">
                        <label for="default-folder"></label>
                        <select id="default-folder" bind:value={settings.default_view_id}>
                            {#each appState.folders as folder}
                                <option value={folder.id}>{folder.name}</option>
                            {/each}
                        </select>
                    </div>
                {:else if settings.default_view_type === 'feed'}
                    <div class="form-group">
                        <label for="default-feed"></label>
                        <select id="default-feed" bind:value={settings.default_view_id}>
                            {#each appState.folders as folder}
                                {#each folder.feeds as feed}
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
                            bind:checked={settings.auto_collapse_folders}
                        >
                    </div>
                </div>

                <div class="form-group">
                    <label for="mark-read-exit">Mark Feed Read on Exit</label>
                    <div class="checkbox-wrap">
                        <input
                            type="checkbox"
                            id="mark-read-exit"
                            bind:checked={settings.mark_feed_read_on_exit}
                        >
                    </div>
                </div>

                <h4 class="section-label">Typography</h4>

                <div class="form-group">
                    <label for="title-font">Article Title Font</label>
                    <input
                        type="text"
                        id="title-font"
                        bind:value={settings.article_title_font}
                        placeholder="Default (Serif)"
                        onclick={() => pickFont('title')}
                    >
                </div>

                <div class="form-group">
                    <label for="body-font">Article Body Font</label>
                    <input
                        type="text"
                        id="body-font"
                        bind:value={settings.article_body_font}
                        placeholder="Default (Sans)"
                        onclick={() => pickFont('body')}
                    >
                </div>

                <h4 class="section-label">Colors</h4>

                <div class="form-group">
                    <label for="title-color">Article Title FG</label>
                    <div class="color-input-wrap">
                        <input
                            type="color"
                            id="title-color"
                            bind:value={settings.article_title_color}
                        >
                        <input type="text" bind:value={settings.article_title_color}>
                    </div>
                </div>

                <div class="form-group">
                    <label for="body-color">Article Body FG</label>
                    <div class="color-input-wrap">
                        <input
                            type="color"
                            id="body-color"
                            bind:value={settings.article_body_color}
                        >
                        <input type="text" bind:value={settings.article_body_color}>
                    </div>
                </div>

                <div class="form-group">
                    <label for="bg-color">Article Body BG</label>
                    <div class="color-input-wrap">
                        <input type="color" id="bg-color" bind:value={settings.article_bg_color}>
                        <input type="text" bind:value={settings.article_bg_color}>
                    </div>
                </div>

                <h4 class="section-label">Display</h4>

                <div class="form-group">
                    <label for="thumb-size">Thumbnail Size</label>
                    <div class="range-wrap">
                        <input
                            type="range"
                            id="thumb-size"
                            min="0"
                            max="156"
                            step="12"
                            bind:value={settings.thumbnail_size}
                        >
                        <span class="range-value"
                            >{settings.thumbnail_size > 0 ? `${settings.thumbnail_size}px` : 'Off'}</span
                        >
                    </div>
                </div>

                <h4 class="section-label">Maintenance</h4>

                <div class="form-group">
                    <label for="retention-days">Auto-delete articles</label>
                    <div class="range-wrap">
                        <input
                            type="range"
                            id="retention-days"
                            min="0"
                            max="365"
                            step="1"
                            bind:value={settings.article_retention_days}
                        >
                        <span class="range-value"
                            >{settings.article_retention_days > 0 ? `${settings.article_retention_days} days` : 'Never'}</span
                        >
                    </div>
                </div>

                <hr>

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
    min-width: 460px;
    max-width: 640px;
    max-height: 80vh;
    overflow: auto;
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
    padding: 0.75rem 1rem;
    overflow-y: auto;
}

.form-container {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
}

.form-group {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.25rem;
}

.form-group label {
    flex: 0 0 180px;
    font-size: 0.9rem;
    color: var(--text-secondary);
    text-align: right;
}

select {
    max-height: 300px;
    overflow-y: auto;
}

.form-group input:not([type="color"]),
.form-group select {
    flex: 1;
    padding: 8px;
    border: 1px solid var(--border-color);
    background: var(--bg-app);
    color: var(--text-primary);
    border-radius: 4px;
    box-sizing: border-box;
    min-width: 160px;
}

.checkbox-wrap {
    display: flex;
    align-items: center;
}

.form-group input[type="text"]#title-font,
.form-group input[type="text"]#body-font {
    cursor: pointer;
}

.checkbox-wrap input[type="checkbox"] {
    width: 16px;
    height: 16px;
    cursor: pointer;
    accent-color: var(--bg-selected);
}

.range-wrap {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
}

.range-wrap input[type="range"] {
    flex: 1;
    accent-color: var(--bg-selected);
    cursor: pointer;
}

.range-value {
    font-size: 0.85rem;
    color: var(--text-secondary);
    min-width: 36px;
    text-align: right;
}

.section-label {
    margin: 0.35rem 0 0;
    font-size: 0.8rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 0.25rem;
}

.color-input-wrap {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
}

.color-input-wrap input[type="color"] {
    width: 32px;
    height: 32px;
    padding: 2px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--bg-app);
    cursor: pointer;
    flex: 0 0 auto;
}

hr {
    border: none;
    border-top: 1px solid var(--border-color);
    margin: 0.5rem 0;
}

.color-input-wrap input[type="text"] {
    flex: 1;
    padding: 8px;
    border: 1px solid var(--border-color);
    background: var(--bg-app);
    color: var(--text-primary);
    border-radius: 4px;
    box-sizing: border-box;
    font-family: monospace;
    font-size: 0.8rem;
}
</style>
