<script lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { tooltip } from "$lib/actions/tooltip.svelte";
import Modal from "./Modal.svelte";

let { isOpen = $bindable(false), onClose }: { isOpen: boolean; onClose: () => void } = $props();

interface AppInfo {
  version: string;
  data_path: string;
  logs_path: string;
  db_path: string;
}

let appInfo = $state<AppInfo>({
  version: "...",
  data_path: "",
  logs_path: "",
  db_path: "",
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

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
}

async function openLogsDir() {
  if (appInfo.logs_path) await openPath(appInfo.logs_path);
}
</script>

<Modal {isOpen} onclose={onClose} width="420px" class="about-modal">
  <div class="modal-body">
    <img src="/feedmee.png" alt="FeedMee" class="logo">
    <h2>FeedMee</h2>
    <p class="tagline">Clean, fast RSS &amp; Atom reading.</p>

    <div class="info-rows">
      <div class="info-row">
        <span class="label">Version</span>
        <span class="value mono bold">{appInfo.version}</span>
        <span class="spacer"></span>
      </div>

      <div class="info-row">
        <span class="label">Data</span>
        <span class="value mono truncate" title={appInfo.data_path}>{appInfo.data_path}</span>
        <button
          type="button"
          class="copy-btn"
          onclick={() => copyToClipboard(appInfo.data_path)}
          use:tooltip={'Copy path'}
        >
          Copy
        </button>
      </div>

      <div class="info-row">
        <span class="label">Logs</span>
        <span class="value mono truncate" title={appInfo.logs_path}>{appInfo.logs_path}</span>
        <button
          type="button"
          class="copy-btn"
          onclick={() => copyToClipboard(appInfo.logs_path)}
          use:tooltip={'Copy path'}
        >
          Copy
        </button>
      </div>

      <div class="info-row">
        <span class="label">Database</span>
        <span class="value mono truncate" title={appInfo.db_path}>{appInfo.db_path}</span>
        <button
          type="button"
          class="copy-btn"
          onclick={() => copyToClipboard(appInfo.db_path)}
          use:tooltip={'Copy path'}
        >
          Copy
        </button>
      </div>
    </div>

    <button type="button" class="open-logs-btn" onclick={openLogsDir}>Open Logs Folder</button>

    <p class="footer">Giants' Shoulders = Rust / Tauri / Svelte / SQLite</p>
    <p class="footer">FeedMee © 2025. All rights reserved.</p>
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
}

.logo {
  width: 64px;
  height: 64px;
  margin-bottom: 0.5rem;
}

h2 {
  margin: 0;
  font-size: 1.4rem;
  font-weight: 700;
  color: var(--text-primary);
}

.tagline {
  margin: 0 0 1rem;
  color: var(--text-secondary);
  font-size: 0.9rem;
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
  font-family: monospace;
}
.bold {
  font-weight: 700;
}

.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.spacer {
  flex: 0;
}

.copy-btn {
  flex-shrink: 0;
  background: none;
  border: none;
  color: var(--bg-selected);
  font-size: 0.75rem;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
}

.copy-btn:hover {
  background: var(--bg-hover);
}

.open-logs-btn {
  background: none;
  border: none;
  color: var(--bg-selected);
  font-size: 0.85rem;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  margin-bottom: 0.5rem;
}

.open-logs-btn:hover {
  background: var(--bg-hover);
}

.footer {
  margin: 0;
  font-size: 0.72rem;
  color: var(--text-secondary);
  opacity: 0.6;
}
</style>
