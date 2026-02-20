<script lang="ts">
    import { tooltip } from '$lib/actions/tooltip.svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { openPath } from '@tauri-apps/plugin-opener';

    let { isOpen = $bindable(false), onClose }: { isOpen: boolean; onClose: () => void } = $props();

    interface AppInfo {
        version: string;
        data_path: string;
        logs_path: string;
        db_path: string;
    }

    let appInfo = $state<AppInfo>({
        version: '...',
        data_path: '',
        logs_path: '',
        db_path: '',
    });

    $effect(() => {
        if (isOpen) {
            invoke<AppInfo>('get_app_info')
                .then((info) => {
                    appInfo = info;
                })
                .catch(console.error);
        }
    });

    function copyToClipboard(text: string) {
        navigator.clipboard.writeText(text);
    }

    async function openLogsDir() {
        if (appInfo.logs_path) await openPath(appInfo.logs_path);
    }

    function onKeyDown(e: KeyboardEvent) {
        if (e.key === 'Escape') onClose();
    }
</script>

<svelte:window onkeydown={onKeyDown} />

{#if isOpen}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="overlay" onclick={onClose} role="presentation">
        <div
            class="modal"
            onclick={(e) => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
            aria-label="About FeedMee"
            tabindex="-1">
            <div class="modal-body">
                <img src="/feedmee.png" alt="FeedMee" class="logo" />
                <h2>FeedMee</h2>
                <p class="tagline">Clean, fast RSS &amp; Atom reading.</p>

                <div class="info-rows">
                    <div class="info-row">
                        <span class="label">Version</span>
                        <span class="value mono bold">{appInfo.version}</span>
                        <span class="spacer"></span>
                    </div>

                    <div class="info-row">
                        <span class="label">Data</span>
                        <span class="value mono truncate" title={appInfo.data_path}
                            >{appInfo.data_path}</span>
                        <button
                            class="copy-btn"
                            onclick={() => copyToClipboard(appInfo.data_path)}
                            use:tooltip={'Copy path'}>Copy</button>
                    </div>

                    <div class="info-row">
                        <span class="label">Logs</span>
                        <span class="value mono truncate" title={appInfo.logs_path}
                            >{appInfo.logs_path}</span>
                        <button
                            class="copy-btn"
                            onclick={() => copyToClipboard(appInfo.logs_path)}
                            use:tooltip={'Copy path'}>Copy</button>
                    </div>

                    <div class="info-row">
                        <span class="label">Database</span>
                        <span class="value mono truncate" title={appInfo.db_path}
                            >{appInfo.db_path}</span>
                        <button
                            class="copy-btn"
                            onclick={() => copyToClipboard(appInfo.db_path)}
                            use:tooltip={'Copy path'}>Copy</button>
                    </div>
                </div>

                <button class="open-logs-btn" onclick={openLogsDir}>Open Logs Folder</button>

                <p class="footer">Giants' Shoulders = Rust / Tauri / Svelte / SQLite</p>
                <p class="footer">FeedMee � 2025. All rights reserved.</p>
            </div>
        </div>
    </div>
{/if}

<style>
    .overlay {
        position: fixed;
        inset: 0;
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
        width: 420px;
        box-shadow: 0 16px 40px rgba(0, 0, 0, 0.25);
        overflow: hidden;
    }

    .modal-body {
        display: flex;
        flex-direction: column;
        align-items: center;
        padding: 2rem;
        gap: 0.5rem;
    }

    .logo {
        width: 64px;
        height: 64px;
        margin-bottom: 0.5rem;
    }

    h2 {
        margin: 0;
        font-size: 1.4rem;
        font-weight: 700;
        color: var(--text-primary);
    }

    .tagline {
        margin: 0 0 1rem;
        color: var(--text-secondary);
        font-size: 0.9rem;
    }

    .info-rows {
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 6px;
        margin-bottom: 1rem;
    }

    .info-row {
        display: flex;
        align-items: center;
        gap: 10px;
        background: var(--bg-app);
        border-radius: 6px;
        padding: 8px 12px;
        min-width: 0;
    }

    .label {
        font-size: 0.8rem;
        font-weight: 600;
        color: var(--text-secondary);
        width: 64px;
        flex-shrink: 0;
    }

    .value {
        flex: 1;
        font-size: 0.8rem;
        color: var(--text-primary);
        min-width: 0;
    }

    .mono {
        font-family: monospace;
    }
    .bold {
        font-weight: 700;
    }

    .truncate {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .spacer {
        flex: 0;
    }

    .copy-btn {
        flex-shrink: 0;
        background: none;
        border: none;
        color: var(--bg-selected);
        font-size: 0.75rem;
        cursor: pointer;
        padding: 2px 6px;
        border-radius: 4px;
    }

    .copy-btn:hover {
        background: var(--bg-hover);
    }

    .open-logs-btn {
        background: none;
        border: none;
        color: var(--bg-selected);
        font-size: 0.85rem;
        cursor: pointer;
        padding: 4px 8px;
        border-radius: 4px;
        margin-bottom: 0.5rem;
    }

    .open-logs-btn:hover {
        background: var(--bg-hover);
    }

    .footer {
        margin: 0;
        font-size: 0.72rem;
        color: var(--text-secondary);
        opacity: 0.6;
    }
</style>
