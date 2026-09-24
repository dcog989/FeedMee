import { invoke } from "@tauri-apps/api/core";
import type { SortOrder, Theme } from "./storeTypes";
import type { AppSettings } from "./types";
import {
  LS_BLOCKED_PHRASES,
  LS_LIST_WIDTH,
  LS_NAV_WIDTH,
  LS_SORT_ORDER,
  LS_THEME,
  writeInt,
  writeJson,
  writeString,
} from "./utils/persistence";

export interface EditFeedTarget {
  id: number;
  name: string;
  source_type: string;
  source_id: string;
}

export interface RenameFolderTarget {
  id: number;
  name: string;
}

export interface ModalState {
  isOpen: boolean;
  type: "confirm" | "alert";
  message: string;
  onConfirm: () => void;
}

export interface UIState {
  showSettings: boolean;
  showAddDialog: boolean;
  showAbout: boolean;
  showNewFolderDialog: boolean;
  showEditFeedDialog: boolean;
  editFeedTarget: EditFeedTarget | null;
  renameFolderTarget: RenameFolderTarget | null;
  modalState: ModalState;
  navWidth: number;
  listWidth: number;
}

export function createUI(state: {
  focusedPane: "nav" | "list" | "reading";
  blockedPhrases: string[];
  theme: Theme;
  sortOrder: SortOrder;
  settings: AppSettings;
  searchQuery: string;
  reloadCurrentArticleList(options?: { selectTop?: boolean }): Promise<void>;
  refreshAllFeeds(): Promise<void>;
}) {
  const ui = $state<UIState>({
    showSettings: false,
    showAddDialog: false,
    showAbout: false,
    showNewFolderDialog: false,
    showEditFeedDialog: false,
    editFeedTarget: null,
    renameFolderTarget: null,
    modalState: {
      isOpen: false,
      type: "confirm",
      message: "",
      onConfirm: () => {},
    },
    navWidth: 280,
    listWidth: 320,
  });

  let autoRefreshTimer: ReturnType<typeof setInterval> | null = null;

  async function setBlockedPhrases(phrases: string[]) {
    state.blockedPhrases = phrases;
    writeJson(LS_BLOCKED_PHRASES, phrases);
    await state.reloadCurrentArticleList();
  }

  function persistLayoutSettings() {
    writeInt(LS_NAV_WIDTH, ui.navWidth);
    writeInt(LS_LIST_WIDTH, ui.listWidth);
    writeString(LS_SORT_ORDER, state.sortOrder);
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
    writeString(LS_THEME, newTheme);
  }

  function openSettings() {
    ui.showSettings = true;
  }

  function closeSettings() {
    ui.showSettings = false;
  }

  function openAbout() {
    ui.showAbout = true;
  }

  function closeAbout() {
    ui.showAbout = false;
  }

  async function saveSettings(newSettings: AppSettings, closeModal = true) {
    const settingsToSave = { ...newSettings };
    try {
      await invoke("save_app_settings", { newSettings: settingsToSave });
      state.settings = settingsToSave;
      startAutoRefreshTimer();
      if (closeModal) closeSettings();
    } catch (e) {
      alert(`Failed to save settings: ${e}`);
    }
  }

  function confirm(message: string, onConfirm: () => void | Promise<void>) {
    ui.modalState = {
      isOpen: true,
      type: "confirm",
      message,
      onConfirm: () => {
        ui.modalState = { ...ui.modalState, isOpen: false };
        Promise.resolve(onConfirm()).catch((e) => console.error("confirm callback failed:", e));
      },
    };
  }

  function alert(message: string) {
    ui.modalState = {
      isOpen: true,
      type: "alert",
      message,
      onConfirm: () => {
        ui.modalState = { ...ui.modalState, isOpen: false };
      },
    };
  }

  function closeModal() {
    ui.modalState = { ...ui.modalState, isOpen: false };
  }

  function startAutoRefreshTimer() {
    if (autoRefreshTimer !== null) {
      clearInterval(autoRefreshTimer);
      autoRefreshTimer = null;
    }
    if (state.settings.auto_update_interval_minutes > 0) {
      const intervalMs = state.settings.auto_update_interval_minutes * 60 * 1000;
      autoRefreshTimer = setInterval(() => state.refreshAllFeeds(), intervalMs);
    }
  }

  return {
    ui,
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
