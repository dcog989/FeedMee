import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { Article, Folder } from "./types";
import { collapseAllFolders, expandAllFolders } from "./utils/expandedFolders";
import { shortcutManager } from "./utils/shortcuts";

interface ShortcutRegDeps {
  showAddDialog: boolean;
  folders: Folder[];
  expandedFolders: Set<number>;
  selectedArticle: Article | null;
  openSettings(): void;
  refreshAllFeeds(): Promise<void>;
  toggleSaved(article: Article): Promise<void>;
  adjustUnreadCount(feedId: number, delta: number): void;
}

export function registerShortcuts(state: ShortcutRegDeps) {
  shortcutManager.register({
    command: "settings",
    defaultKey: ",",
    description: "Open settings",
    category: "General",
    handler: () => state.openSettings(),
  });

  shortcutManager.register({
    command: "add-feed",
    defaultKey: "n",
    description: "Add new feed",
    category: "General",
    handler: () => {
      state.showAddDialog = true;
    },
  });

  shortcutManager.register({
    command: "refresh-all",
    defaultKey: "r",
    description: "Refresh all feeds",
    category: "Feeds",
    handler: () => state.refreshAllFeeds(),
  });

  shortcutManager.register({
    command: "focus-search",
    defaultKey: "/",
    description: "Focus search",
    category: "General",
    handler: () => {
      const searchInput = document.querySelector(".search-wrapper input") as HTMLInputElement;
      searchInput?.focus();
    },
  });

  shortcutManager.register({
    command: "toggle-save",
    defaultKey: "s",
    description: "Save/Read later",
    category: "Articles",
    handler: () => {
      if (state.selectedArticle) state.toggleSaved(state.selectedArticle);
    },
  });

  shortcutManager.register({
    command: "mark-read",
    defaultKey: "m",
    description: "Mark as read/unread",
    category: "Articles",
    handler: async () => {
      if (!state.selectedArticle) return;
      const article = state.selectedArticle;
      const newReadState = !article.is_read;
      article.is_read = newReadState;
      state.adjustUnreadCount(article.feed_id, newReadState ? -1 : 1);
      try {
        await invoke("mark_article_read", {
          id: article.id,
          read: newReadState,
        });
      } catch (e) {
        article.is_read = !newReadState;
        state.adjustUnreadCount(article.feed_id, newReadState ? 1 : -1);
        console.error("mark_article_read failed:", e);
      }
    },
  });

  shortcutManager.register({
    command: "expand-all",
    defaultKey: "x",
    description: "Expand all folders",
    category: "Feeds",
    handler: () => {
      state.expandedFolders = expandAllFolders(state.folders);
    },
  });

  shortcutManager.register({
    command: "collapse-all",
    defaultKey: "c",
    description: "Collapse all folders",
    category: "Feeds",
    handler: () => {
      state.expandedFolders = collapseAllFolders();
    },
  });

  shortcutManager.register({
    command: "open-article",
    defaultKey: "enter",
    description: "Open article in browser",
    category: "Articles",
    handler: () => {
      if (state.selectedArticle) openUrl(state.selectedArticle.url);
    },
  });
}

interface KeyHandlerDeps {
  showSettings: boolean;
  focusedPane: "nav" | "list" | "reading";
  selectedArticle: Article | null;
  navUp(): void;
  navDown(): void;
  articleUp(): void;
  articleDown(): void;
}

let activeKeyHandler: ((e: KeyboardEvent) => void) | null = null;

export function setupKeyHandler(state: KeyHandlerDeps): () => void {
  // Remove any previously bound handler (e.g. after a hot-reload that
  // re-instantiates the store) so keydown listeners don't stack.
  if (activeKeyHandler) {
    window.removeEventListener("keydown", activeKeyHandler);
    activeKeyHandler = null;
  }

  const handler = (e: KeyboardEvent) => {
    if (state.showSettings) return;

    const tag = (e.target as HTMLElement)?.tagName?.toLowerCase();
    const isInput = tag === "input" || tag === "textarea" || (e.target as HTMLElement)?.isContentEditable;
    if (isInput) return;

    switch (e.key) {
      case "ArrowLeft":
        e.preventDefault();
        if (state.focusedPane === "reading") state.focusedPane = "list";
        else if (state.focusedPane === "list") state.focusedPane = "nav";
        return;
      case "ArrowRight":
        e.preventDefault();
        if (state.focusedPane === "nav") state.focusedPane = "list";
        else if (state.focusedPane === "list" && state.selectedArticle) state.focusedPane = "reading";
        return;
      case "ArrowUp":
        e.preventDefault();
        if (state.focusedPane === "nav") state.navUp();
        else if (state.focusedPane === "list") state.articleUp();
        else if (state.focusedPane === "reading") {
          document.querySelector<HTMLElement>(".reading-area .pane")?.scrollBy({ top: -80, behavior: "smooth" });
        }
        return;
      case "ArrowDown":
        e.preventDefault();
        if (state.focusedPane === "nav") state.navDown();
        else if (state.focusedPane === "list") state.articleDown();
        else if (state.focusedPane === "reading") {
          document.querySelector<HTMLElement>(".reading-area .pane")?.scrollBy({ top: 80, behavior: "smooth" });
        }
        return;
    }

    shortcutManager.handleKeyEvent(e);
  };

  activeKeyHandler = handler;
  window.addEventListener("keydown", handler);

  return () => {
    window.removeEventListener("keydown", handler);
    if (activeKeyHandler === handler) activeKeyHandler = null;
  };
}
