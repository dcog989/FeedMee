<script lang="ts">
import { appState } from '$lib/store.svelte';

let name = $state(appState.editFeedTarget?.name ?? '');
let url = $state(appState.editFeedTarget?.url ?? '');

function closeDialog() {
    appState.showEditFeedDialog = false;
    appState.editFeedTarget = null;
}

function submit() {
    if (name.trim() && url.trim() && appState.editFeedTarget) {
        appState.renameFeed(appState.editFeedTarget.id, name.trim(), url.trim());
    }
    closeDialog();
}

function onKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' && name.trim() && url.trim()) {
        submit();
    } else if (e.key === 'Escape') {
        closeDialog();
    }
}

function focusOnMount(node: HTMLInputElement) {
    node.focus();
    node.select();
}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- biome-ignore lint/a11y/noStaticElementInteractions: backdrop is decorative, can't use button because it wraps modal with buttons -->
<div
    class="modal-overlay"
    role="presentation"
    onclick={(e) => { if (e.target === e.currentTarget) closeDialog(); }}
>
    <div
        class="modal"
        role="dialog"
        aria-modal="true"
        tabindex="-1"
    >
        <h3>Edit Feed</h3>
        <label class="field">
            <span>Name</span>
            <input
                type="text"
                bind:value={name}
                placeholder="Feed name"
                onkeydown={onKeyDown}
                use:focusOnMount
            >
        </label>
        <label class="field">
            <span>URL</span>
            <input type="text" bind:value={url} placeholder="Feed URL" onkeydown={onKeyDown}>
        </label>
        <div class="modal-actions">
            <button type="button" class="secondary" onclick={closeDialog}>Cancel</button>
            <button
                type="button"
                class="primary"
                disabled={!name.trim() || !url.trim()}
                onclick={submit}
            >
                Save
            </button>
        </div>
    </div>
</div>

<style>
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
    width: 420px;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-color);
}

.modal h3 {
    margin: 0 0 1rem 0;
    font-size: 1.1rem;
    color: var(--text-primary);
}

.field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 0.75rem;
}

.field span {
    font-size: 0.85rem;
    color: var(--text-secondary);
}

.field input {
    width: 100%;
    padding: 8px 12px;
    background: var(--bg-app);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    border-radius: 4px;
    font-size: 0.9rem;
    outline: none;
    box-sizing: border-box;
}

.field input:focus {
    border-color: var(--bg-selected);
}

.modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 1rem;
}

button {
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 500;
    border: none;
}

button.secondary {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-primary);
}

button.secondary:hover {
    background: var(--bg-hover);
}

button.primary {
    background: var(--bg-selected);
    color: white;
}

button.primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

button.primary:not(:disabled):hover {
    opacity: 0.9;
}
</style>
