<script lang="ts">
import { Keyboard, Settings, X } from "@lucide/svelte";
import { invoke } from "@tauri-apps/api/core";
import { articleStore, feedStore, settingsStore, uiStore } from "#lib/store.svelte.js";
import type { AppSettings } from "#lib/types.js";
import ArticleAppearanceSettings from "./ArticleAppearanceSettings.svelte";
import Modal from "./Modal.svelte";
import ShortcutsModal from "./ShortcutsModal.svelte";

let settings = $state({ ...settingsStore.settings });
let showShortcuts = $state(false);
let initialized = $state(false);
let prevSettings = $state<AppSettings | null>(null);
let saveTimer: ReturnType<typeof setTimeout> | null = null;

const COLOR_KEYS = ["article_title_color", "article_body_color", "article_bg_color"] as const;

/** Colours an empty setting falls back to in the current theme. */
function themeColors(): Record<(typeof COLOR_KEYS)[number], string> {
  const cs = getComputedStyle(document.documentElement);
  return {
    article_title_color: cs.getPropertyValue("--accent-muted").trim(),
    article_body_color: cs.getPropertyValue("--text-primary").trim(),
    article_bg_color: cs.getPropertyValue("--bg-reading").trim(),
  };
}

$effect(() => {
  const s = settingsStore.settings;
  if (s && "default_view_type" in s) {
    const theme = themeColors();
    settings = {
      ...s,
      article_title_color: s.article_title_color || theme.article_title_color,
      article_body_color: s.article_body_color || theme.article_body_color,
      article_bg_color: s.article_bg_color || theme.article_bg_color,
    };
    if (!initialized) {
      initialized = true;
      prevSettings = { ...settings };
    }
  }
});

function settingsEqual(a: AppSettings, b: AppSettings): boolean {
  for (const key of Object.keys(a) as (keyof AppSettings)[]) {
    if (a[key] !== b[key]) return false;
  }
  return true;
}

$effect(() => {
  if (!initialized || !prevSettings) return;
  if (settingsEqual(settings, prevSettings)) return;
  // A number field is briefly undefined while it is being retyped; saving that
  // fails deserialisation on the Rust side and shows a spurious error alert.
  if (!Number.isFinite(settings.auto_update_interval_minutes)) return;
  prevSettings = { ...settings };
  if (saveTimer) clearTimeout(saveTimer);
  // Empty means "follow the theme". The form shows the resolved colour, so
  // strip anything that came from the theme instead of pinning it on save.
  const theme = themeColors();
  const toSave = { ...settings };
  toSave.auto_update_interval_minutes = Math.max(0, Math.floor(toSave.auto_update_interval_minutes));
  for (const key of COLOR_KEYS) {
    if (toSave[key].toLowerCase() === theme[key].toLowerCase()) toSave[key] = "";
  }
  saveTimer = setTimeout(() => settingsStore.saveSettings(toSave, false), 500);
});

// Switching between "Folder…" and "Feed…" must not keep an id from the other list.
$effect(() => {
  const type = settings.default_view_type;
  const ids =
    type === "folder"
      ? feedStore.folders.filter((folder) => folder.id !== 0).map((folder) => folder.id)
      : type === "feed"
        ? feedStore.folders.flatMap((folder) => folder.feeds.map((feed) => feed.id))
        : null;
  if (!ids || ids.includes(settings.default_view_id)) return;
  settings = { ...settings, default_view_id: ids[0] ?? -1 };
});

function cancel() {
  (document.activeElement as HTMLElement)?.blur();
  settingsStore.closeSettings();
}

interface StorageStats {
  db_size_bytes: number;
  cache_size_bytes: number;
  cache_file_count: number;
}

let storage = $state<StorageStats | null>(null);
let clearing = $state(false);
let vacuuming = $state(false);

async function refreshStorage() {
  try {
    storage = await invoke<StorageStats>("get_storage_stats");
  } catch (e) {
    console.error("Failed to load storage stats", e);
  }
}

$effect(() => {
  refreshStorage();
});

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB"];
  let value = bytes / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return `${value.toFixed(value >= 10 ? 0 : 1)} ${units[unit]}`;
}

function cacheLabel(): string {
  if (!storage) return "…";
  const files = storage.cache_file_count;
  return `${formatBytes(storage.cache_size_bytes)} · ${files} ${files === 1 ? "file" : "files"}`;
}

function dbLabel(): string {
  return storage ? formatBytes(storage.db_size_bytes) : "…";
}

async function clearCache() {
  if (clearing) return;
  clearing = true;
  try {
    await invoke("clear_thumbnail_cache");
    articleStore.invalidateThumbnails();
    await refreshStorage();
  } catch (e) {
    uiStore.alert(String(e));
  } finally {
    clearing = false;
  }
}

function vacuum() {
  uiStore.confirm("Vacuum the database now? This may take a moment.", async () => {
    vacuuming = true;
    try {
      await invoke("vacuum_database");
      await refreshStorage();
    } catch (e) {
      uiStore.alert(String(e));
    } finally {
      vacuuming = false;
    }
  });
}
</script>

<Modal isOpen={true} onclose={cancel} label="Settings" class="settings-modal">
  <div class="modal-header">
    <h3><Settings size={18} /> Settings</h3>
    <button type="button" class="shortcuts-btn" onclick={() => (showShortcuts = true)} title="Keyboard Shortcuts">
      <Keyboard size={18} />
    </button>
    <button type="button" class="close-btn" onclick={cancel} title="Close">
      <X size={18} />
    </button>
  </div>

  <div class="modal-content">
    <div class="form-container">
      <h4 class="section-label">General</h4>

      <div class="form-group">
        <label for="auto-update">Auto Update Interval</label>
        <input
          type="number"
          id="auto-update"
          bind:value={settings.auto_update_interval_minutes}
          min="0"
          step="1"
          title="0 to disable"
        >
        <span class="unit">minutes</span>
      </div>

      <div class="form-group">
        <label for="default-view">View on Startup</label>
        <select id="default-view" bind:value={settings.default_view_type}>
          <option value="latest">Latest</option>
          <option value="saved">Read Later</option>
          <option value="last">Last Folder or Feed</option>
          <option value="folder">Folder…</option>
          <option value="feed">Feed…</option>
        </select>
      </div>

      {#if settings.default_view_type === "folder"}
        <div class="form-group">
          <label for="default-folder">Default Folder</label>
          <select id="default-folder" bind:value={settings.default_view_id}>
            {#each feedStore.folders.filter((f) => f.id !== 0) as folder}
              <option value={folder.id}>{folder.name}</option>
            {/each}
          </select>
        </div>
      {:else if settings.default_view_type === "feed"}
        <div class="form-group">
          <label for="default-feed">Default Feed</label>
          <select id="default-feed" bind:value={settings.default_view_id}>
            {#each feedStore.folders.filter((f) => f.id !== 0) as folder}
              {#each folder.feeds as feed}
                <option value={feed.id}>{folder.name} / {feed.name}</option>
              {/each}
            {/each}
          </select>
        </div>
      {/if}

      <div class="form-group">
        <label for="startup-article">Article on Startup</label>
        <select id="startup-article" bind:value={settings.startup_article}>
          <option value="latest">Latest article</option>
          <option value="last">Last opened article</option>
        </select>
      </div>

      <div class="form-group">
        <label for="mark-read-exit">Mark Feed Read on Exit</label>
        <div class="checkbox-wrap">
          <input type="checkbox" id="mark-read-exit" bind:checked={settings.mark_feed_read_on_exit}>
        </div>
      </div>

      <hr>

      <h4 class="section-label">Visuals</h4>

      <ArticleAppearanceSettings {settings} onchange={(next) => (settings = next)} />

      <div class="form-group">
        <label for="thumb-size">Thumbnail Size</label>
        <div class="range-wrap">
          <input type="range" id="thumb-size" min="0" max="156" step="12" bind:value={settings.thumbnail_size}>
          <span class="range-value">{settings.thumbnail_size > 0 ? `${settings.thumbnail_size}px` : "Off"}</span>
        </div>
      </div>

      <hr>

      <h4 class="section-label">Data</h4>

      <div class="form-group">
        <label for="retention-days">Auto-delete articles</label>
        <div class="range-wrap">
          <input
            type="range"
            id="retention-days"
            min="0"
            max="365"
            step="1"
            bind:value={settings.article_retention_days}
          >
          <span class="range-value"
            >{settings.article_retention_days > 0 ? `${settings.article_retention_days} days` : "Never"}</span
          >
        </div>
      </div>

      <div class="form-group">
        <span class="field-label">Thumbnail Cache</span>
        <div class="storage-wrap">
          <span class="storage-size">{cacheLabel()}</span>
          <button type="button" class="storage-btn" onclick={clearCache} disabled={clearing || storage === null}>
            {clearing ? "Clearing…" : "Clear"}
          </button>
        </div>
      </div>

      <div class="form-group">
        <span class="field-label">Database</span>
        <div class="storage-wrap">
          <span class="storage-size">{dbLabel()}</span>
          <button type="button" class="storage-btn" onclick={vacuum} disabled={vacuuming}>
            {vacuuming ? "Vacuuming…" : "Vacuum"}
          </button>
        </div>
      </div>
    </div>
  </div>
</Modal>

<ShortcutsModal bind:isOpen={showShortcuts} onClose={() => (showShortcuts = false)} />

<style>
:global(.settings-modal) {
  min-width: 460px;
  max-width: 640px;
  padding: 0;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.modal-header h3 {
  margin: 0;
  flex: 1;
  font-size: 1rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.shortcuts-btn,
.close-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
}

.shortcuts-btn:hover,
.close-btn:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.modal-content {
  padding: 0.75rem 1rem;
  overflow-y: auto;
}

.form-container {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  min-width: 0;
}

:global(.settings-modal .form-group) {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.25rem;
  padding-left: 0.75rem;
}

:global(.settings-modal .form-group label),
:global(.settings-modal .form-group .field-label) {
  flex: 0 0 180px;
  font-size: 0.9rem;
  color: var(--text-secondary);
  text-align: left;
}

.unit {
  font-size: 0.85rem;
  color: var(--text-secondary);
  flex: none;
}

.storage-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.storage-size {
  flex: 1;
  font-size: 0.85rem;
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
}

.storage-btn {
  flex: 0 0 auto;
  padding: 6px 12px;
  border: 1px solid var(--border-color);
  background: var(--bg-app);
  color: var(--text-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8rem;
}

.storage-btn:hover:not(:disabled) {
  background: var(--bg-hover);
}

.storage-btn:disabled {
  opacity: 0.5;
  cursor: default;
}

select {
  max-height: 300px;
  overflow-y: auto;
}

:global(.settings-modal .form-group input:not([type="color"])),
:global(.settings-modal .form-group select) {
  flex: 1;
  padding: 8px;
  border: 1px solid var(--border-color);
  background: var(--bg-app);
  color: var(--text-primary);
  border-radius: 4px;
  box-sizing: border-box;
  min-width: 160px;
}

.checkbox-wrap {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  flex: 1;
  padding-left: 8px;
}

.checkbox-wrap input[type="checkbox"] {
  width: 16px;
  height: 16px;
  flex: none;
  padding: 0;
  border: none;
  background: none;
  min-width: 0;
  cursor: pointer;
  accent-color: var(--bg-selected);
}

.range-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.range-wrap input[type="range"] {
  flex: 1;
  accent-color: var(--bg-selected);
  cursor: pointer;
}

.range-value {
  font-size: 0.85rem;
  color: var(--text-secondary);
  min-width: 36px;
  text-align: right;
}

:global(.settings-modal .section-label) {
  margin: 0.35rem 0 0;
  font-size: 0.8rem;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

:global(.settings-modal .section-label:first-child) {
  margin-top: 0;
}

:global(.settings-modal hr) {
  border: none;
  border-top: 1px solid var(--border-color);
  margin: 0.5rem 0;
  width: 100%;
}
</style>
