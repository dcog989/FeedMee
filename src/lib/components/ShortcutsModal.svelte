<script lang="ts">
import { Keyboard, RotateCcw, X } from "@lucide/svelte";
import { shortcutStore } from "#lib/store.svelte.js";
import { type ShortcutDefinition, shortcutKeyFromEvent, shortcutManager } from "#lib/utils/shortcuts.js";
import Modal from "./Modal.svelte";

let { isOpen = $bindable(false), onClose }: { isOpen: boolean; onClose: () => void } = $props();

let recordingCommandId = $state<string | null>(null);

const MODIFIER_KEYS = ["Control", "Shift", "Alt", "Meta"];

// The dialog stays mounted, so closing it must also disarm the recorder.
$effect(() => {
  if (!isOpen) {
    recordingCommandId = null;
    return;
  }
  if (!recordingCommandId) return;
  window.addEventListener("keydown", handleRecordKey, { capture: true });
  return () => window.removeEventListener("keydown", handleRecordKey, { capture: true });
});

const allShortcuts = $derived(shortcutManager.getDefinitions());

const conflicts = $derived(shortcutManager.findConflicts(shortcutStore.customShortcuts));

const descriptions = $derived(new Map(allShortcuts.map((def) => [def.command, def.description])));

const categories = $derived.by(() => {
  const map = new Map<string, ShortcutDefinition[]>();
  allShortcuts.forEach((def) => {
    if (!map.has(def.category)) map.set(def.category, []);
    map.get(def.category)?.push(def);
  });
  return Array.from(map.entries());
});

function startRecording(commandId: string) {
  recordingCommandId = commandId;
}

function handleRecordKey(e: KeyboardEvent) {
  if (!recordingCommandId) return;
  e.preventDefault();
  e.stopPropagation();

  if (e.key === "Escape") {
    stopRecording();
    return;
  }

  // Modifiers alone and the keys the app reserves for pane navigation can't be
  // bound: the global handler always consumes the arrows first.
  if (MODIFIER_KEYS.includes(e.key) || e.key.startsWith("Arrow") || e.key === "Tab") return;

  // Same normaliser as the matcher, or a recorded binding never fires.
  shortcutStore.setShortcut(recordingCommandId, shortcutKeyFromEvent(e));
  stopRecording();
}

function stopRecording() {
  recordingCommandId = null;
}

function resetShortcut(commandId: string) {
  shortcutStore.resetShortcut(commandId);
}
</script>

<Modal {isOpen} onclose={onClose} zindex={10001} label="Keyboard Shortcuts" class="shortcuts-modal">
  <div class="modal-header">
    <h3><Keyboard size={18} /> Keyboard Shortcuts</h3>
    <button type="button" class="close-btn" onclick={onClose} aria-label="Close">
      <X size={18} />
    </button>
  </div>

  <div class="shortcuts-list">
    {#if conflicts.length > 0}
      <div class="conflict-warning" role="status">
        {#each conflicts as conflict (conflict.key)}
          <p>
            <strong>{conflict.key}</strong>
            is bound to
            {conflict.commands.map((command) => descriptions.get(command)).join(", ")}
            — only the first one runs.
          </p>
        {/each}
      </div>
    {/if}

    {#each categories as [category, defs] (category)}
      <div class="section">
        <h4>{category}</h4>
        {#each defs as def (def.command)}
          {@const isRecording = recordingCommandId === def.command}
          {@const hasCustom = shortcutStore.customShortcuts[def.command]}
          <div class="shortcut-row">
            <span class="description">{def.description}</span>
            <div class="shortcut-actions">
              <button
                type="button"
                class="shortcut-key"
                class:recording={isRecording}
                onclick={() => startRecording(def.command)}
                title="Click to change"
              >
                {isRecording ? "Press keys..." : shortcutManager.getShortcutDisplay(def.command)}
              </button>
              {#if hasCustom}
                <button
                  type="button"
                  class="reset-btn"
                  onclick={() => resetShortcut(def.command)}
                  title="Reset to default"
                >
                  <RotateCcw size={14} />
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/each}
  </div>
</Modal>

<style>
:global(.shortcuts-modal) {
  width: 440px;
  max-height: 80vh;
  padding: 0;
  overflow: hidden;
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
  margin-bottom: 0;
}

.shortcuts-list {
  padding: 1rem 1.25rem;
  overflow-y: auto;
  flex: 1;
}

.conflict-warning {
  margin-bottom: 1rem;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--border-color);
  border-left: 3px solid var(--accent-muted);
  border-radius: 4px;
}

.conflict-warning p {
  margin: 0;
  font-size: 0.85rem;
  color: var(--text-secondary);
}

.conflict-warning strong {
  color: var(--text-primary);
}

.section {
  margin-bottom: 1.25rem;
}

.section:last-child {
  margin-bottom: 0;
}

h4 {
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-secondary);
  margin: 0 0 0.5rem 0;
  font-weight: 600;
}

.shortcut-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.4rem 0;
}

.description {
  font-size: 0.85rem;
  color: var(--text-primary);
}

.shortcut-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.shortcut-key {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  background: var(--bg-app);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 3px 8px;
  color: var(--text-primary);
  cursor: pointer;
  min-width: 70px;
  text-align: center;
}

.shortcut-key:hover {
  border-color: var(--bg-selected);
}

.shortcut-key.recording {
  border-color: var(--bg-selected);
  background: var(--bg-selected);
  color: white;
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.7;
  }
}

.reset-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 2px;
  display: flex;
  border-radius: 4px;
}

.reset-btn:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}
</style>
