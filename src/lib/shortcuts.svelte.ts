import { invoke } from '@tauri-apps/api/core';
import type { AppState } from './storeTypes';
import { shortcutManager } from './utils/shortcuts';

export function createShortcutOps(state: AppState) {
  function setShortcut(commandId: string, key: string) {
    state.customShortcuts = { ...state.customShortcuts, [commandId]: key };
    shortcutManager.setCustomMappings(state.customShortcuts);
    saveShortcutSettings();
  }

  function resetShortcut(commandId: string) {
    const updated = { ...state.customShortcuts };
    delete updated[commandId];
    state.customShortcuts = updated;
    shortcutManager.setCustomMappings(state.customShortcuts);
    saveShortcutSettings();
  }

  async function saveShortcutSettings() {
    try {
      await invoke('save_shortcuts', { shortcuts: state.customShortcuts });
    } catch (e) {
      console.error('Failed to save shortcuts:', e);
    }
  }

  async function loadShortcutSettings() {
    try {
      const shortcuts = await invoke<Record<string, string>>('get_shortcuts');
      state.customShortcuts = shortcuts || {};
      shortcutManager.setCustomMappings(state.customShortcuts);
    } catch (e) {
      console.error('Failed to load shortcuts:', e);
    }
  }

  return {
    setShortcut,
    resetShortcut,
    loadShortcutSettings,
  };
}
