<script lang="ts">
import { feedStore, uiStore } from '$lib/store.svelte';
import ContextMenu from '../ContextMenu.svelte';

let cmVisible = $state(false);
let cmX = $state(0);
let cmY = $state(0);
let cmTarget = $state<{
  type: 'folder' | 'feed' | 'root';
  id: number;
  name?: string;
} | null>(null);

export function show(event: MouseEvent, type: 'folder' | 'feed' | 'root', id: number, name?: string) {
  event.preventDefault();
  event.stopPropagation();
  cmVisible = true;
  cmX = event.clientX;
  cmY = event.clientY;
  cmTarget = { type, id, name };
}

export function close() {
  cmVisible = false;
  cmTarget = null;
}

function rename() {
  if (cmTarget?.type !== 'folder') return;
  uiStore.renameFolderTarget = { id: cmTarget.id, name: cmTarget.name ?? '' };
  uiStore.showNewFolderDialog = true;
  close();
}

function renameFeed() {
  if (cmTarget?.type !== 'feed') return;
  const feed = feedStore.folders.flatMap((f) => f.feeds).find((f) => f.id === cmTarget?.id);
  uiStore.editFeedTarget = {
    id: cmTarget?.id,
    name: cmTarget?.name ?? '',
    source_type: feed?.source_type ?? '',
    source_id: feed?.source_id ?? '',
  };
  uiStore.showEditFeedDialog = true;
  close();
}

function deleteTarget() {
  if (!cmTarget) return;
  if (cmTarget.type === 'folder') {
    feedStore.deleteFolder(cmTarget.id);
  } else if (cmTarget.type === 'feed') {
    feedStore.deleteFeed(cmTarget.id);
  }
  close();
}

function createFolder() {
  uiStore.showNewFolderDialog = true;
  close();
}
</script>

<ContextMenu x={cmX} y={cmY} visible={cmVisible} onClose={close}>
  {#if cmTarget?.type === 'root'}
    <button type="button" onclick={createFolder}>New Folder</button>
  {:else if cmTarget?.type === 'folder'}
    <button type="button" onclick={rename}>Rename Folder</button>
    <button type="button" class="danger" onclick={deleteTarget}>Delete Folder</button>
  {:else if cmTarget?.type === 'feed'}
    <button type="button" onclick={renameFeed}>Edit Feed</button>
    <button type="button" class="danger" onclick={deleteTarget}>Delete Feed</button>
  {/if}
</ContextMenu>
