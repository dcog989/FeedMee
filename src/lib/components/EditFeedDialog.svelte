<script lang="ts">
import { feedStore, uiStore } from '$lib/store.svelte';
import Modal from './Modal.svelte';

let name = $state(uiStore.editFeedTarget?.name ?? '');
let url = $state(uiStore.editFeedTarget?.url ?? '');
let isBluesky = $derived(url.startsWith('bsky:'));

function closeDialog() {
    uiStore.showEditFeedDialog = false;
    uiStore.editFeedTarget = null;
}

function submit() {
    if (name.trim() && url.trim() && uiStore.editFeedTarget) {
        feedStore.renameFeed(uiStore.editFeedTarget.id, name.trim(), url.trim());
    }
    closeDialog();
}

function onKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' && name.trim() && url.trim()) {
        submit();
    }
}

function focusOnMount(node: HTMLInputElement) {
    node.focus();
    node.select();
}
</script>

<Modal isOpen={true} onclose={closeDialog} width="420px" class="edit-feed-dialog">
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
        <input type="text" bind:value={url} placeholder="Feed URL" onkeydown={onKeyDown} disabled={isBluesky}>
        {#if isBluesky}
            <span class="field-note">Bluesky feed URL is fixed and cannot be edited.</span>
        {/if}
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
</Modal>

<style>
h3 {
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

.field input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.field-note {
    font-size: 0.8rem;
    color: var(--text-secondary);
    margin-top: 2px;
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
