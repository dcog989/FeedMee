import { invoke } from "@tauri-apps/api/core";
import type { SortOrder, Theme } from "./storeTypes";
import type { AppSettings } from "./types";
import { LS_BLOCKED_PHRASES, LS_LIST_WIDTH, LS_NAV_WIDTH, LS_SORT_ORDER, LS_THEME } from "./utils/persistence";

export function createUI(state: {
  showSettings: boolean;
  showAddDialog: boolean;
  showAbout: boolean;
  showNewFolderDialog: boolean;
  showEditFeedDialog: boolean;
  editFeedTarget: { id: number; name: string; source_type: string; source_id: string } | null;
  renameFolderTarget: { id: number; name: string } | null;
  focusedPane: "nav" | "list" | "reading";
  modalState: {
    isOpen: boolean;
    type: "confirm" | "alert";
    message: string;
    onConfirm: () => void;
  };
  blockedPhrases: string[];
  navWidth: number;
  listWidth: number;
  theme: Theme;
  sortOrder: SortOrder;
  settings: AppSettings;
  searchQuery: string;
  autoRefreshTimer: ReturnType<typeof setInterval> | null;
  reloadCurrentArticleList(options?: { selectTop?: boolean }): Promise<void>;
  refreshAllFeeds(): Promise<void>;
}) {
  async function setBlockedPhrases(phrases: string[]) {
    state.blockedPhrases = phrases;
    localStorage.setItem(LS_BLOCKED_PHRASES, JSON.stringify(phrases));
    await state.reloadCurrentArticleList();
  }

  function persistLayoutSettings() {
    localStorage.setItem(LS_NAV_WIDTH, state.navWidth.toString());
    localStorage.setItem(LS_LIST_WIDTH, state.listWidth.toString());
    localStorage.setItem(LS_SORT_ORDER, state.sortOrder);
  }

  async function setSortOrder(order: SortOrder) {
    if (state.sortOrder !== order) {
      state.sortOrder = order;
      persistLayoutSettings();
      await state.reloadCurrentArticleList();
    }
  }

  async function setSearch(query: string) {
    state.searchQuery = query;
    await state.reloadCurrentArticleList();
  }

  function setTheme(newTheme: Theme) {
    state.theme = newTheme;
    localStorage.setItem(LS_THEME, newTheme);
  }

  function openSettings() {
    state.showSettings = true;
  }

  function closeSettings() {
    state.showSettings = false;
  }

  function openAbout() {
    state.showAbout = true;
  }

  function closeAbout() {
    state.showAbout = false;
  }

  async function saveSettings(newSettings: AppSettings, closeModal = true) {
    try {
      await invoke("save_app_settings", { newSettings });
      state.settings = newSettings;
      if (state.autoRefreshTimer !== null) {
        clearInterval(state.autoRefreshTimer);
        state.autoRefreshTimer = null;
      }
      startAutoRefreshTimer();
      if (closeModal) closeSettings();
    } catch (e) {
      alert(`Failed to save settings: ${e}`);
    }
  }

  function confirm(message: string, onConfirm: () => void | Promise<void>) {
    state.modalState = {
      isOpen: true,
      type: "confirm",
      message,
      onConfirm: () => {
        state.modalState = { ...state.modalState, isOpen: false };
        Promise.resolve(onConfirm()).catch((e) => console.error("confirm callback failed:", e));
      },
    };
  }

  function alert(message: string) {
    state.modalState = {
      isOpen: true,
      type: "alert",
      message,
      onConfirm: () => {
        state.modalState = { ...state.modalState, isOpen: false };
      },
    };
  }

  function closeModal() {
    state.modalState = { ...state.modalState, isOpen: false };
  }

  function startAutoRefreshTimer() {
    if (state.settings.auto_update_interval_minutes > 0) {
      const intervalMs = state.settings.auto_update_interval_minutes * 60 * 1000;
      state.autoRefreshTimer = setInterval(() => state.refreshAllFeeds(), intervalMs);
    }
  }

  return {
    setBlockedPhrases,
    persistLayoutSettings,
    setSortOrder,
    setSearch,
    setTheme,
    openSettings,
    closeSettings,
    openAbout,
    closeAbout,
    saveSettings,
    confirm,
    alert,
    closeModal,
    startAutoRefreshTimer,
  };
}
