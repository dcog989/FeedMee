<script lang="ts">
import { feedStore, uiStore } from "$lib/store.svelte";
import Modal from "./Modal.svelte";

let isRename = $derived(!!uiStore.renameFolderTarget);
let name = $state(uiStore.renameFolderTarget?.name ?? "");

function closeDialog() {
  uiStore.showNewFolderDialog = false;
  uiStore.renameFolderTarget = null;
}

function submit() {
  if (!name.trim()) return;
  if (isRename && uiStore.renameFolderTarget) {
    feedStore.renameFolder(uiStore.renameFolderTarget.id, name.trim());
  } else {
    feedStore.createFolder(name.trim());
  }
  closeDialog();
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Enter" && name.trim()) {
    submit();
  }
}

function focusOnMount(node: HTMLInputElement) {
  node.focus();
  if (isRename) node.select();
}
</script>

<Modal isOpen={true} onclose={closeDialog} width="350px">
  <h3>{isRename ? 'Rename Folder' : 'New Folder'}</h3>
  <input type="text" bind:value={name} placeholder="Enter folder name" onkeydown={onKeyDown} use:focusOnMount>
  <div class="modal-actions">
    <button type="button" class="secondary" onclick={closeDialog}>Cancel</button>
    <button type="button" class="primary" disabled={!name.trim()} onclick={submit}>
      {isRename ? 'Rename' : 'Create'}
    </button>
  </div>
</Modal>

<style>
h3 {
  margin: 0 0 1rem 0;
  font-size: 1.1rem;
  color: var(--text-primary);
}

input {
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

input:focus {
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
