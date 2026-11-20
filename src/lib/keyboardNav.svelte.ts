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
  setArticleRead(article: Article, read: boolean): void;
}

export function registerShortcuts(state: ShortcutRegDeps) {
  shortcutManager.register({
    command: "settings",
    defaultKey: "ctrl+,",
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
    handler: () => {
      if (!state.selectedArticle) return;
      state.setArticleRead(state.selectedArticle, !state.selectedArticle.is_read);
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

const ARROW_SCROLL_PX = 80;
const PAGE_SCROLL_RATIO = 0.9;

function scrollReadingPane(delta: number) {
  document.querySelector<HTMLElement>(".reading-area .pane")?.scrollBy({ top: delta, behavior: "smooth" });
}

export function setupKeyHandler(state: KeyHandlerDeps): void {
  // Remove any previously bound handler (e.g. after a hot-reload that
  // re-instantiates the store) so keydown listeners don't stack.
  if (activeKeyHandler) {
    window.removeEventListener("keydown", activeKeyHandler);
    activeKeyHandler = null;
  }

  const handler = (e: KeyboardEvent) => {
    if (state.showSettings) return;
    // An open dialog owns the keyboard; shortcuts must not act behind it.
    if (document.querySelector(".modal-overlay")) return;

    const tag = (e.target as HTMLElement)?.tagName?.toLowerCase();
    const isInput =
      tag === "input" || tag === "textarea" || tag === "select" || (e.target as HTMLElement)?.isContentEditable;
    if (isInput) return;

    // A held arrow re-selects feeds in the nav pane, rerunning the article query
    // on every repeat. The list and reading panes only move the selection or scroll.
    if (e.repeat && state.focusedPane === "nav") return;

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
        else if (state.focusedPane === "reading") scrollReadingPane(-ARROW_SCROLL_PX);
        return;
      case "ArrowDown":
        e.preventDefault();
        if (state.focusedPane === "nav") state.navDown();
        else if (state.focusedPane === "list") state.articleDown();
        else if (state.focusedPane === "reading") scrollReadingPane(ARROW_SCROLL_PX);
        return;
      case "PageUp":
      case "PageDown":
        if (state.focusedPane !== "reading") return;
        e.preventDefault();
        scrollReadingPane((e.key === "PageUp" ? -1 : 1) * window.innerHeight * PAGE_SCROLL_RATIO);
        return;
    }

    shortcutManager.handleKeyEvent(e);
  };

  activeKeyHandler = handler;
  window.addEventListener("keydown", handler);
}
