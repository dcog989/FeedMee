<script lang="ts">
import { appState } from '$lib/store.svelte';

let newFeedUrl = $state('');
let selectedFolderId = $state<number | null>(null);
let isSubmitting = $state(false);
let errorMessage = $state('');
let successMessage = $state('');

let isValidUrl = $derived(/^https?:\/\/.+/.test(newFeedUrl.trim()));
let canSubmit = $derived(isValidUrl && !isSubmitting);

$effect(() => {
    if (!appState.showAddDialog) return;
    newFeedUrl = '';
    selectedFolderId = null;
    isSubmitting = false;
    errorMessage = '';
    successMessage = '';
    try {
        navigator.clipboard.readText().then((text) => {
            if (newFeedUrl === '' && /^https?:\/\/.+/.test(text.trim())) {
                newFeedUrl = text.trim();
            }
        });
    } catch {
        /* clipboard access denied */
    }
});

function closeDialog() {
    appState.showAddDialog = false;
}

async function submitAddFeed() {
    if (!canSubmit) return;
    isSubmitting = true;
    errorMessage = '';
    successMessage = '';
    try {
        await appState.addFeed(newFeedUrl.trim(), selectedFolderId);
        successMessage = 'Feed added successfully';
        await new Promise((r) => setTimeout(r, 1200));
        closeDialog();
    } catch (e) {
        errorMessage = String(e);
    } finally {
        isSubmitting = false;
    }
}

function handleImport() {
    appState.importOpml();
    closeDialog();
}

async function handleExport() {
    await appState.exportOpml();
    closeDialog();
}

function onKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' && canSubmit) {
        submitAddFeed();
    } else if (e.key === 'Escape') {
        closeDialog();
    }
}

function focusOnMount(node: HTMLElement) {
    node.focus();
}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-overlay" onclick={closeDialog}>
    <div class="modal" onclick={(e) => e.stopPropagation()}>
        <h3>Manage Content</h3>
        <div class="input-group">
            <input
                type="text"
                bind:value={newFeedUrl}
                placeholder="Feed URL or Bluesky profile URL"
                onkeydown={onKeyDown}
                use:focusOnMount
                disabled={isSubmitting}
            >
            <button type="button" class="primary" disabled={!canSubmit} onclick={submitAddFeed}>
                {#if successMessage}
                    Added!
                {:else if isSubmitting}
                    Adding Feed...
                {:else}
                    Add Feed
                {/if}
            </button>
        </div>

        <div class="hint">Supports RSS/Atom feeds, websites, and Bluesky profiles</div>

        {#if successMessage}
            <div class="success-message">{successMessage}</div>
        {:else if errorMessage}
            <div class="error-message">{errorMessage}</div>
        {/if}

        <div class="form-group">
            <label for="folder-select">Add to folder</label>
            <select id="folder-select" bind:value={selectedFolderId}>
                <option value={null}>Uncategorized</option>
                {#each appState.folders as folder (folder.id)}
                    <option value={folder.id}>{folder.name}</option>
                {/each}
            </select>
        </div>

        <div class="divider">
            <span>OR</span>
        </div>

        <div class="opml-row">
            <button type="button" class="secondary" onclick={handleImport}>Import OPML File</button>
            <button type="button" class="secondary" onclick={handleExport}>Export OPML File</button>
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
    width: 400px;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-color);
}

.modal h3 {
    margin: 0 0 1rem 0;
    font-size: 1.1rem;
    color: var(--text-primary);
}

.input-group {
    display: flex;
    gap: 8px;
    margin-bottom: 1rem;
}

.input-group input {
    flex: 1;
    padding: 8px 12px;
    background: var(--bg-app);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    border-radius: 4px;
    font-size: 0.9rem;
    outline: none;
}

.input-group input:focus {
    border-color: var(--bg-selected);
}

button.primary {
    background-color: var(--bg-selected);
    color: white;
    border: none;
    border-radius: 4px;
    padding: 0 12px;
    font-weight: 500;
    cursor: pointer;
}

button.primary:hover {
    opacity: 0.9;
}

button.primary:disabled {
    background-color: var(--bg-hover, #555);
    color: var(--text-secondary, #999);
    cursor: not-allowed;
    opacity: 0.6;
}

.opml-row {
    display: flex;
    gap: 8px;
}

.opml-row button {
    flex: 1;
}

button.secondary {
    padding: 8px;
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    border-radius: 4px;
    cursor: pointer;
}

button.secondary:hover {
    background-color: var(--bg-hover);
}

.form-group {
    margin-bottom: 1rem;
}

.form-group label {
    display: block;
    font-size: 0.8rem;
    color: var(--text-secondary);
    margin-bottom: 4px;
}

.form-group select {
    width: 100%;
    padding: 8px;
    background: var(--bg-app);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    border-radius: 4px;
    font-size: 0.9rem;
    cursor: pointer;
}

.hint {
    font-size: 0.75rem;
    color: var(--text-secondary, #888);
    margin-bottom: 0.75rem;
    padding: 0 2px;
}

.success-message {
    color: #2ecc71;
    font-size: 0.85rem;
    margin-bottom: 0.75rem;
    padding: 6px 10px;
    background: rgba(46, 204, 113, 0.1);
    border-radius: 4px;
}

.error-message {
    color: var(--text-danger, #e74c3c);
    font-size: 0.85rem;
    margin-bottom: 0.75rem;
    padding: 6px 10px;
    background: rgba(231, 76, 60, 0.1);
    border-radius: 4px;
    word-break: break-word;
}

.divider {
    display: flex;
    align-items: center;
    text-align: center;
    margin: 1rem 0;
    color: var(--text-secondary);
    font-size: 0.8rem;
}

.divider::before,
.divider::after {
    content: "";
    flex: 1;
    border-bottom: 1px solid var(--border-color);
}

.divider span {
    padding: 0 10px;
}
</style>
