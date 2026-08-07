<script lang="ts">
import AboutModal from '$lib/components/AboutModal.svelte';
import ConfirmModal from '$lib/components/ConfirmModal.svelte';
import EditFeedDialog from '$lib/components/EditFeedDialog.svelte';
import ManageDialog from '$lib/components/ManageDialog.svelte';
import NewFolderDialog from '$lib/components/NewFolderDialog.svelte';
import SettingsModal from '$lib/components/SettingsModal.svelte';
import StyleInjector from '$lib/components/StyleInjector.svelte';
import ThemeManager from '$lib/components/ThemeManager.svelte';
import Tooltip from '$lib/components/Tooltip.svelte';
import { appState } from '$lib/store.svelte';
import '../app.css';

let { children } = $props();

function disableContextMenu(e: MouseEvent) {
  e.preventDefault();
}
</script>

<div class="layout-wrapper" role="application" oncontextmenu={disableContextMenu}>
  <div class="content-wrapper">
    {@render children()}
  </div>
  <ThemeManager />
  <StyleInjector />
  <Tooltip />

  {#if appState.showSettings}
    <SettingsModal />
  {/if}

  {#if appState.showAddDialog}
    <ManageDialog />
  {/if}

  {#if appState.showAbout}
    <AboutModal bind:isOpen={appState.showAbout} onClose={() => (appState.showAbout = false)} />
  {/if}

  {#if appState.showNewFolderDialog}
    <NewFolderDialog />
  {/if}

  {#if appState.showEditFeedDialog}
    <EditFeedDialog />
  {/if}

  <ConfirmModal />
</div>

<style>
.layout-wrapper {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

.content-wrapper {
  flex: 1;
  overflow: hidden;
  position: relative;
}
</style>
