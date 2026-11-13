<script lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onMount } from "svelte";
import AboutModal from "#lib/components/AboutModal.svelte";
import ConfirmModal from "#lib/components/ConfirmModal.svelte";
import EditFeedDialog from "#lib/components/EditFeedDialog.svelte";
import ManageDialog from "#lib/components/ManageDialog.svelte";
import NewFolderDialog from "#lib/components/NewFolderDialog.svelte";
import SettingsModal from "#lib/components/SettingsModal.svelte";
import StyleInjector from "#lib/components/StyleInjector.svelte";
import ThemeManager from "#lib/components/ThemeManager.svelte";
import Tooltip from "#lib/components/Tooltip.svelte";
import { uiStore } from "#lib/store.svelte.js";
import "../app.css";

let { children } = $props();

// The window starts hidden (visible: false) and is only shown once the theme
// (data-theme) has been applied, preventing a white flash on startup.
onMount(() => {
  invoke("show_main_window");
});

function disableContextMenu(e: MouseEvent) {
  // Text fields keep the native menu, so copy and paste stay reachable.
  if ((e.target as HTMLElement | null)?.closest("input, textarea")) return;
  e.preventDefault();
}
</script>

<div class="layout" role="application" oncontextmenu={disableContextMenu}>
  {@render children()}
  <ThemeManager />
  <StyleInjector />
  <Tooltip />

  {#if uiStore.showSettings}
    <SettingsModal />
  {/if}

  {#if uiStore.showAddDialog}
    <ManageDialog />
  {/if}

  {#if uiStore.showAbout}
    <AboutModal isOpen onClose={() => (uiStore.showAbout = false)} />
  {/if}

  {#if uiStore.showNewFolderDialog}
    <NewFolderDialog />
  {/if}

  {#if uiStore.showEditFeedDialog}
    <EditFeedDialog />
  {/if}

  <ConfirmModal />
</div>

<style>
.layout {
  position: relative;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}
</style>
