<script lang="ts">
import { X } from "@lucide/svelte";
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { tooltip } from "#lib/actions/tooltip.svelte.js";
import Modal from "./Modal.svelte";

let { isOpen = $bindable(false), onClose }: { isOpen: boolean; onClose: () => void } = $props();

interface AppInfo {
  version: string;
  data_path: string;
  logs_path: string;
  backup_path: string;
  settings_path: string;
}

let appInfo = $state<AppInfo>({
  version: "...",
  data_path: "",
  logs_path: "",
  backup_path: "",
  settings_path: "",
});

$effect(() => {
  if (isOpen) {
    invoke<AppInfo>("get_app_info")
      .then((info) => {
        appInfo = info;
      })
      .catch(console.error);
  }
});

async function openPathSafe(path: string) {
  if (!path) return;
  try {
    await openPath(path);
  } catch (e) {
    console.error("Failed to open path:", e);
  }
}
</script>

<Modal {isOpen} onclose={onClose} width="640px" label="About FeedMee" class="about-modal">
  <div class="modal-body">
    <button type="button" class="close-btn" onclick={onClose} aria-label="Close">
      <X size={18} />
    </button>
    <img src="/feedmee.png" alt="FeedMee" class="logo">
    <h2>FeedMee</h2>
    <p class="tagline">Clean, fast RSS &amp; Atom reading.</p>
    <p class="version">Version: {appInfo.version}</p>

    <div class="info-rows">
      <div class="info-row">
        <span class="label">Config</span>
        <button
          type="button"
          class="value mono path"
          onclick={() => openPathSafe(appInfo.settings_path)}
          use:tooltip={"Open settings file"}
        >
          {appInfo.settings_path}
        </button>
      </div>

      <div class="info-row">
        <span class="label">Data</span>
        <button
          type="button"
          class="value mono path"
          onclick={() => openPathSafe(appInfo.data_path)}
          use:tooltip={"Open data folder"}
        >
          {appInfo.data_path}
        </button>
      </div>

      <div class="info-row">
        <span class="label">Logs</span>
        <button
          type="button"
          class="value mono path"
          onclick={() => openPathSafe(appInfo.logs_path)}
          use:tooltip={"Open logs folder"}
        >
          {appInfo.logs_path}
        </button>
      </div>

      <div class="info-row">
        <span class="label" use:tooltip={"Automatic subscription backups (OPML)"}>Backups</span>
        <button
          type="button"
          class="value mono path"
          onclick={() => openPathSafe(appInfo.backup_path)}
          use:tooltip={"Open backups folder"}
        >
          {appInfo.backup_path}
        </button>
      </div>
    </div>

    <p class="footer">Giants' Shoulders = Rust / Tauri / Svelte / SQLite</p>
    <p class="footer">FeedMee is free software under the GNU GPL v3.0 or later.</p>
    <p class="footer">© 2025 dcog989</p>
  </div>
</Modal>

<style>
:global(.about-modal) {
  padding: 0;
  overflow: hidden;
}

.modal-body {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 2rem;
  gap: 0.5rem;
  position: relative;
}

.logo {
  width: 64px;
  height: 64px;
  margin-bottom: 0.5rem;
}

.close-btn {
  position: absolute;
  top: 0.75rem;
  right: 0.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  border: none;
  border-radius: 6px;
  background: none;
  color: var(--text-secondary);
  cursor: pointer;
  transition:
    background-color 0.15s,
    color 0.15s;
}

.close-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

h2 {
  margin: 0;
  font-size: 1.4rem;
  font-weight: 700;
  color: var(--text-primary);
}

.tagline {
  margin: 0 0 0.25rem;
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.version {
  margin: 0 0 1rem;
  font-family: var(--font-mono);
  font-size: 0.9rem;
  font-weight: 700;
  color: var(--text-secondary);
}

.info-rows {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 1rem;
}

.info-row {
  display: flex;
  align-items: center;
  gap: 10px;
  background: var(--bg-app);
  border-radius: 6px;
  padding: 8px 12px;
  min-width: 0;
}

.label {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--text-secondary);
  width: 64px;
  flex-shrink: 0;
}

.value {
  flex: 1;
  font-size: 0.8rem;
  color: var(--text-primary);
  min-width: 0;
}

.mono {
  font-family: var(--font-mono);
}

.path {
  background: none;
  border: none;
  padding: 0;
  color: var(--bg-selected);
  text-align: left;
  overflow-wrap: anywhere;
  cursor: pointer;
}

.path:hover {
  text-decoration: underline;
}

.footer {
  margin: 0;
  font-size: 0.72rem;
  color: var(--text-secondary);
  opacity: 0.6;
}
</style>
