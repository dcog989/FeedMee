<script lang="ts">
    import { appState } from '$lib/store.svelte';
    import { shortcutManager, type ShortcutDefinition } from '$lib/utils/shortcuts';
    import { RotateCcw, X } from 'lucide-svelte';

    let { isOpen = $bindable(false), onClose }: { isOpen: boolean; onClose: () => void } = $props();

    let recordingCommandId = $state<string | null>(null);

    $effect(() => {
        if (isOpen) {
            recordingCommandId = null;
        }
    });

    const allShortcuts = $derived(shortcutManager.getDefinitions());

    const categories = $derived(() => {
        const map = new Map<string, ShortcutDefinition[]>();
        allShortcuts.forEach((def) => {
            if (!map.has(def.category)) map.set(def.category, []);
            map.get(def.category)!.push(def);
        });
        return Array.from(map.entries());
    });

    function startRecording(commandId: string) {
        recordingCommandId = commandId;
        window.addEventListener('keydown', handleRecordKey, { capture: true });
    }

    function handleRecordKey(e: KeyboardEvent) {
        if (!recordingCommandId) return;
        e.preventDefault();
        e.stopPropagation();

        if (e.key === 'Escape') {
            stopRecording();
            return;
        }

        if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) return;

        const parts: string[] = [];
        if (e.ctrlKey) parts.push('ctrl');
        if (e.altKey) parts.push('alt');
        if (e.shiftKey) parts.push('shift');
        if (e.metaKey) parts.push('meta');
        parts.push(e.key.toLowerCase());

        const keyStr = parts.join('+');
        appState.setShortcut(recordingCommandId, keyStr);
        stopRecording();
    }

    function stopRecording() {
        recordingCommandId = null;
        window.removeEventListener('keydown', handleRecordKey);
    }

    function resetShortcut(commandId: string) {
        appState.resetShortcut(commandId);
    }

    function onKeyDown(e: KeyboardEvent) {
        if (e.key === 'Escape') {
            if (recordingCommandId) {
                stopRecording();
            } else {
                onClose();
            }
        }
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
            aria-label="Keyboard Shortcuts"
            tabindex="-1">
            <div class="modal-header">
                <h3>Keyboard Shortcuts</h3>
                <button class="close-btn" onclick={onClose} aria-label="Close">
                    <X size={18} />
                </button>
            </div>

            <div class="shortcuts-list">
                {#each categories() as [category, defs] (category)}
                    <div class="section">
                        <h4>{category}</h4>
                        {#each defs as def (def.command)}
                            {@const isRecording = recordingCommandId === def.command}
                            {@const hasCustom = appState.customShortcuts[def.command]}
                            <div class="shortcut-row">
                                <span class="description">{def.description}</span>
                                <div class="shortcut-actions">
                                    <button
                                        class="shortcut-key"
                                        class:recording={isRecording}
                                        onclick={() => startRecording(def.command)}
                                        title="Click to change">
                                        {isRecording
                                            ? 'Press keys...'
                                            : shortcutManager.getShortcutDisplay(def.command)}
                                    </button>
                                    {#if hasCustom}
                                        <button
                                            class="reset-btn"
                                            onclick={() => resetShortcut(def.command)}
                                            title="Reset to default">
                                            <RotateCcw size={14} />
                                        </button>
                                    {/if}
                                </div>
                            </div>
                        {/each}
                    </div>
                {/each}
                {#if allShortcuts.length === 0}
                    <div class="empty-state">
                        <p>No shortcuts match your search</p>
                    </div>
                {/if}
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
        z-index: 10001;
        backdrop-filter: blur(2px);
    }

    .modal {
        background: var(--bg-pane);
        border: 1px solid var(--border-color);
        border-radius: 10px;
        width: 440px;
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
    }

    .close-btn {
        background: none;
        border: none;
        color: var(--text-secondary);
        cursor: pointer;
        padding: 4px;
        display: flex;
        border-radius: 4px;
    }

    .close-btn:hover {
        color: var(--text-primary);
        background: var(--bg-hover);
    }

    .shortcuts-list {
        padding: 1rem 1.25rem;
        overflow-y: auto;
        flex: 1;
    }

    .section {
        margin-bottom: 1.25rem;
    }

    .section:last-child {
        margin-bottom: 0;
    }

    h4 {
        font-size: 0.7rem;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        color: var(--text-secondary);
        margin: 0 0 0.5rem 0;
        font-weight: 600;
    }

    .shortcut-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.4rem 0;
    }

    .description {
        font-size: 0.85rem;
        color: var(--text-primary);
    }

    .shortcut-actions {
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .shortcut-key {
        font-family: var(--font-mono, monospace);
        font-size: 0.75rem;
        background: var(--bg-app);
        border: 1px solid var(--border-color);
        border-radius: 4px;
        padding: 3px 8px;
        color: var(--text-primary);
        cursor: pointer;
        min-width: 70px;
        text-align: center;
    }

    .shortcut-key:hover {
        border-color: var(--bg-selected);
    }

    .shortcut-key.recording {
        border-color: var(--bg-selected);
        background: var(--bg-selected);
        color: white;
        animation: pulse 1s infinite;
    }

    @keyframes pulse {
        0%,
        100% {
            opacity: 1;
        }
        50% {
            opacity: 0.7;
        }
    }

    .reset-btn {
        background: none;
        border: none;
        color: var(--text-secondary);
        cursor: pointer;
        padding: 2px;
        display: flex;
        border-radius: 4px;
    }

    .reset-btn:hover {
        color: var(--text-primary);
        background: var(--bg-hover);
    }

    .empty-state {
        text-align: center;
        padding: 2rem;
        color: var(--text-secondary);
    }
</style>
