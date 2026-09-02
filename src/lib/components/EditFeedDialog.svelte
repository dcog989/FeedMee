<script lang="ts">
import { feedStore, uiStore } from '$lib/store.svelte';
import Modal from './Modal.svelte';

let target = $state(uiStore.editFeedTarget);
let name = $state(target?.name ?? '');
let sourceId = $state(target?.source_id ?? '');
let sourceType = $state(target?.source_type ?? '');
let selectedFolderId = $state<number | null>(getFeedFolderId(target?.id));
let isBluesky = $derived(sourceType === 'bluesky');

function getFeedFolderId(feedId: number | undefined): number | null {
  if (feedId === undefined) return null;
  for (const folder of feedStore.folders) {
    if (folder.feeds.some((f) => f.id === feedId)) return folder.id;
  }
  return null;
}

function closeDialog() {
  uiStore.showEditFeedDialog = false;
  uiStore.editFeedTarget = null;
}

async function submit() {
  if (name.trim() && sourceId.trim() && target) {
    const url = isBluesky ? `bsky:${sourceId.trim()}` : sourceId.trim();
    await feedStore.renameFeed(target.id, name.trim(), url);
    await feedStore.moveFeed(target.id, selectedFolderId === 0 ? null : selectedFolderId);
  }
  closeDialog();
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'Enter' && name.trim() && sourceId.trim()) {
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
    <input type="text" bind:value={name} placeholder="Feed name" onkeydown={onKeyDown} use:focusOnMount>
  </label>
  <div class="field">
    <span id="feed-source-id-label">{isBluesky ? 'Bluesky DID' : 'URL'}</span>
    <input
      aria-labelledby="feed-source-id-label"
      type="text"
      bind:value={sourceId}
      placeholder="Feed URL"
      onkeydown={onKeyDown}
      disabled={isBluesky}
    >
  </div>
  {#if isBluesky}
    <span class="field-note">Bluesky feed identifier is fixed and cannot be edited.</span>
  {/if}
  <div class="field">
    <span id="feed-folder-label">Folder</span>
    <select id="feed-folder-select" aria-labelledby="feed-folder-label" bind:value={selectedFolderId}>
      <option value={0}>Root (no folder)</option>
      {#each feedStore.folders.filter((f) => f.id !== 0) as folder (folder.id)}
        <option value={folder.id}>{folder.name}</option>
      {/each}
    </select>
  </div>
  <div class="modal-actions">
    <button type="button" class="secondary" onclick={closeDialog}>Cancel</button>
    <button type="button" class="primary" disabled={!name.trim() || !sourceId.trim()} onclick={submit}>Save</button>
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

.field select {
  width: 100%;
  padding: 8px 12px;
  background: var(--bg-app);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  border-radius: 4px;
  font-size: 0.9rem;
  cursor: pointer;
  box-sizing: border-box;
}

.field select:focus {
  border-color: var(--bg-selected);
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
