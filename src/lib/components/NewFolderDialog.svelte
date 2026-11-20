<script lang="ts">
import { FolderPen, FolderPlus, X } from "@lucide/svelte";
import { feedStore, uiStore } from "#lib/store.svelte.js";
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

<Modal isOpen={true} onclose={closeDialog} width="350px" label={isRename ? "Rename Folder" : "New Folder"}>
  <div class="modal-header">
    <h3>
      {#if isRename}
        <FolderPen size={18} />
      {:else}
        <FolderPlus size={18} />
      {/if}
      {isRename ? "Rename Folder" : "New Folder"}
    </h3>
    <button type="button" class="close-btn" onclick={closeDialog} aria-label="Close">
      <X size={18} />
    </button>
  </div>
  <input type="text" bind:value={name} placeholder="Enter folder name" onkeydown={onKeyDown} use:focusOnMount>
  <div class="modal-actions">
    <button type="button" class="secondary" onclick={closeDialog}>Cancel</button>
    <button type="button" class="primary" disabled={!name.trim()} onclick={submit}>
      {isRename ? "Rename" : "Create"}
    </button>
  </div>
</Modal>

<style>
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
</style>
