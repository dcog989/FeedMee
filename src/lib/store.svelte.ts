import { invoke } from "@tauri-apps/api/core";
import { createArticleActions } from "./articleActions.svelte";
import { createFeedActions } from "./feedActions.svelte";
import { createFeedRefresher } from "./feedRefresh.svelte";
import { createFreshnessHelpers } from "./freshness.svelte";
import { registerShortcuts, setupKeyHandler } from "./keyboardNav.svelte";
import { createNavigation } from "./navigation.svelte";
import { createShortcutOps } from "./shortcuts.svelte";
import type {
  AppState,
  ArticleStore,
  FeedStore,
  NavStore,
  RefreshStore,
  SettingsStore,
  ShortcutStore,
  SortOrder,
  TagStore,
  Theme,
  UIStore,
} from "./storeTypes";
import { createTagOps } from "./tags.svelte";
import type { AppSettings, Article, Folder } from "./types";
import { DEFAULT_SETTINGS } from "./types";
import type { EditFeedTarget, ModalState, RenameFolderTarget } from "./ui.svelte";
import { createUI } from "./ui.svelte";
import { expandFolder } from "./utils/expandedFolders";
import {
  LS_BLOCKED_PHRASES,
  LS_EXPANDED_FOLDERS,
  LS_LAST_REFRESHED,
  LS_LAST_VIEW_ID,
  LS_LAST_VIEW_TYPE,
  LS_LIST_WIDTH,
  LS_NAV_WIDTH,
  LS_SORT_ORDER,
  LS_THEME,
  readInt,
  readJson,
  readString,
} from "./utils/persistence";
import { pickStartupTarget, startupExpandFolderId } from "./utils/startupView";

export type {
  AppState,
  ArticleStore,
  FeedStore,
  NavStore,
  RefreshStore,
  SettingsStore,
  ShortcutStore,
  TagStore,
  UIStore,
} from "./storeTypes";
export type { Article, SortOrder, Theme };
export const FEED_ID_LATEST = -1;
export const FEED_ID_SAVED = -2;
export const FEED_ID_TODAY = -3;

export function applyThemeToDocument(theme: Theme) {
  const root = document.documentElement;
  if (theme !== "system") {
    root.setAttribute("data-theme", theme);
    return;
  }
  const dark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  root.setAttribute("data-theme", dark ? "dark" : "light");
}

class AppStateImpl {
  folders = $state<Folder[]>([]);
  articles = $state<Article[]>([]);
  selectedFeedId = $state<number | null>(null);
  selectedFolderId = $state<number | null>(null);
  selectedArticle = $state<Article | null>(null);
  isLoadingArticles = $state(false);
  isRefreshingFeeds = $state(false);
  searchQuery = $state("");
  theme = $state<Theme>("system");
  sortOrder = $state<SortOrder>("desc");

  settings = $state<AppSettings>({ ...DEFAULT_SETTINGS });

  expandedFolders = $state<Set<number>>(new Set());
  focusedPane = $state<"nav" | "list" | "reading">("nav");
  blockedPhrases = $state<string[]>([]);
  customShortcuts = $state<Record<string, string>>({});

  showSettings = $state(false);
  showAddDialog = $state(false);
  showAbout = $state(false);
  showNewFolderDialog = $state(false);
  showEditFeedDialog = $state(false);
  editFeedTarget = $state<EditFeedTarget | null>(null);
  renameFolderTarget = $state<RenameFolderTarget | null>(null);
  modalState = $state<ModalState>({
    isOpen: false,
    type: "confirm",
    message: "",
    onConfirm: () => {},
  });
  navWidth = $state(280);
  listWidth = $state(320);

  page = 0;
  readonly pageSize = 50;
  hasMore = $state(true);
  readonly latestHours = 24;

  lastRefreshed = new Map<number, number>();
  updatingFeedIds = $state(new Set<number>());

  private refresh: ReturnType<typeof createFeedRefresher>;
  private feedOps: ReturnType<typeof createFeedActions>;
  private articleOps: ReturnType<typeof createArticleActions>;
  private freshness: ReturnType<typeof createFreshnessHelpers>;
  private nav: ReturnType<typeof createNavigation>;
  private tagOps: ReturnType<typeof createTagOps>;
  private shortcutOps: ReturnType<typeof createShortcutOps>;
  private ui: ReturnType<typeof createUI>;
  constructor() {
    this.refresh = createFeedRefresher(this);
    this.feedOps = createFeedActions(this);
    this.articleOps = createArticleActions(this);
    this.freshness = createFreshnessHelpers(this);
    this.nav = createNavigation(this);
    this.tagOps = createTagOps();
    this.shortcutOps = createShortcutOps(this);
    this.ui = createUI(this);
    registerShortcuts(this);
    setupKeyHandler(this);
    this.initStore().catch((e) => console.error("Store initialization failed:", e));
  }

  get debounceMs() {
    return this.settings.feed_refresh_debounce_minutes * 60 * 1000;
  }

  get articleStore(): ArticleStore {
    return this;
  }

  get feedStore(): FeedStore {
    return this;
  }

  get navStore(): NavStore {
    return this;
  }

  get refreshStore(): RefreshStore {
    return this;
  }

  get uiStore(): UIStore {
    return this;
  }

  get settingsStore(): SettingsStore {
    return this;
  }

  get tagStore(): TagStore {
    return this;
  }

  get shortcutStore(): ShortcutStore {
    return this;
  }

  adjustUnreadCount = (feedId: number, delta: number) => this.freshness.adjustUnreadCount(feedId, delta);
  isFeedUpdating = (feedId: number) => this.freshness.isFeedUpdating(feedId);
  isFolderUpdating = (folderId: number) => this.freshness.isFolderUpdating(folderId);
  persistLastRefreshed = () => this.freshness.persistLastRefreshed();
  isFeedFresh = (feedId: number) => this.freshness.isFeedFresh(feedId);
  isFolderFresh = (folderId: number) => this.freshness.isFolderFresh(folderId);
  isAllFresh = () => this.freshness.isAllFresh();

  selectFeed = (feedId: number) => this.nav.selectFeed(feedId);
  selectFolder = (folderId: number) => this.nav.selectFolder(folderId);
  navUp = () => this.nav.navUp();
  navDown = () => this.nav.navDown();
  articleUp = () => this.nav.articleUp();
  articleDown = () => this.nav.articleDown();

  getArticleTags = (articleId: number) => this.tagOps.getArticleTags(articleId);
  getAllTags = () => this.tagOps.getAllTags();
  addTag = (articleId: number, name: string, color?: string) => this.tagOps.addTag(articleId, name, color);
  removeTag = (articleId: number, tagId: number) => this.tagOps.removeTag(articleId, tagId);
  deleteTag = (tagId: number) => this.tagOps.deleteTag(tagId);

  setShortcut = (commandId: string, key: string) => this.shortcutOps.setShortcut(commandId, key);
  resetShortcut = (commandId: string) => this.shortcutOps.resetShortcut(commandId);

  async refreshFolders() {
    try {
      const result = await invoke<Folder[]>("get_folders_with_feeds");
      this.folders = result || [];
    } catch (e) {
      console.error("Failed to load folders:", e);
    }
  }

  refreshAllFeeds = () => this.refresh.refreshAllFeeds();
  requestRefreshFeed = (feedId: number) => this.refresh.requestRefreshFeed(feedId);
  requestRefreshFolder = (folderId: number) => this.refresh.requestRefreshFolder(folderId);

  reloadCurrentArticleList = () => this.articleOps.reloadCurrentArticleList();
  loadMore = () => this.articleOps.loadMore();
  selectArticle = (article: Article) => this.articleOps.selectArticle(article);
  toggleSaved = (article: Article) => this.articleOps.toggleSaved(article);
  setArticleRead = (article: Article, read: boolean) => this.articleOps.setArticleRead(article, read);
  fetchFullContent = (article: Article) => this.articleOps.fetchFullContent(article);

  markAllRead = () => this.feedOps.markAllRead();
  addFeed = (url: string, folderId?: number | null) => this.feedOps.addFeed(url, folderId);
  createFolder = (name: string) => this.feedOps.createFolder(name);
  importOpml = () => this.feedOps.importOpml();
  exportOpml = () => this.feedOps.exportOpml();
  renameFolder = (id: number, newName: string) => this.feedOps.renameFolder(id, newName);
  renameFeed = (id: number, newName: string, newUrl: string) => this.feedOps.renameFeed(id, newName, newUrl);
  deleteFeed = (id: number) => this.feedOps.deleteFeed(id);
  deleteFolder = (id: number) => this.feedOps.deleteFolder(id);
  moveFeed = (feedId: number, folderId: number | null) => this.feedOps.moveFeed(feedId, folderId);

  setBlockedPhrases = (phrases: string[]) => this.ui.setBlockedPhrases(phrases);
  persistLayoutSettings = () => this.ui.persistLayoutSettings();
  setSortOrder = (order: SortOrder) => this.ui.setSortOrder(order);
  setSearch = (query: string) => this.ui.setSearch(query);
  setTheme = (newTheme: Theme) => this.ui.setTheme(newTheme);
  openSettings = () => this.ui.openSettings();
  closeSettings = () => this.ui.closeSettings();
  openAbout = () => this.ui.openAbout();
  closeAbout = () => this.ui.closeAbout();
  saveSettings = (newSettings: AppSettings, closeModal?: boolean) => this.ui.saveSettings(newSettings, closeModal);
  confirm = (message: string, onConfirm: () => void | Promise<void>) => this.ui.confirm(message, onConfirm);
  alert = (message: string) => this.ui.alert(message);
  closeModal = () => this.ui.closeModal();

  private async initStore() {
    this.navWidth = readInt(LS_NAV_WIDTH, this.navWidth);
    this.listWidth = readInt(LS_LIST_WIDTH, this.listWidth);

    const storedSort = readString(LS_SORT_ORDER);
    const storedTheme = readString(LS_THEME);
    if (storedSort === "asc" || storedSort === "desc") this.sortOrder = storedSort;
    if (storedTheme === "light" || storedTheme === "dark" || storedTheme === "system") this.theme = storedTheme;
    applyThemeToDocument(this.theme);

    const storedPhrases = readJson<unknown>(LS_BLOCKED_PHRASES, []);
    this.blockedPhrases = Array.isArray(storedPhrases)
      ? storedPhrases.filter((phrase): phrase is string => typeof phrase === "string")
      : [];

    // Storage can hold anything (hand edits, old formats); a wrong shape must
    // not abort startup.
    const storedExpandedFolders = readJson<unknown>(LS_EXPANDED_FOLDERS, null);
    const storedFolderIds = Array.isArray(storedExpandedFolders)
      ? storedExpandedFolders.filter((id): id is number => typeof id === "number")
      : null;

    const storedLastRefreshed = readJson<Record<string, number> | null>(LS_LAST_REFRESHED, null);
    if (storedLastRefreshed !== null) {
      this.lastRefreshed = new Map(Object.entries(storedLastRefreshed).map(([k, v]) => [parseInt(k, 10), v]));
    }

    await Promise.all([
      invoke<AppSettings>("get_app_settings")
        .then((s) => {
          this.settings = s;
        })
        .catch((e) => console.error("Failed to load settings", e)),
      this.shortcutOps.loadShortcutSettings(),
    ]);
    this.ui.startAutoRefreshTimer();

    await this.refreshFolders();

    const target = pickStartupTarget(this.settings, this.folders, {
      type: readString(LS_LAST_VIEW_TYPE),
      id: readInt(LS_LAST_VIEW_ID, 0),
    });

    if (target.kind === "saved") await this.selectFeed(FEED_ID_SAVED);
    else if (target.kind === "feed") await this.selectFeed(target.id);
    else if (target.kind === "folder") await this.selectFolder(target.id);
    else await this.selectFeed(FEED_ID_LATEST);

    // `expandedFolders` has a single owner here; the persistence effect in
    // useExpandedFolders only writes the current value back to storage. When
    // startup targets a specific folder, honor auto-collapse so a previously
    // expanded folder does not linger alongside it.
    const expandFolderId = startupExpandFolderId(target, this.folders);
    const restored = new Set<number>(storedFolderIds ?? this.folders.map((f) => f.id));
    this.expandedFolders =
      expandFolderId !== null ? expandFolder(restored, expandFolderId, this.settings.auto_collapse_folders) : restored;

    // Establish the initial view first so the refresh can publish the selected
    // folder's list as soon as its own feeds finish, rather than after the whole
    // batch completes.
    this.refreshAllFeeds();
  }
}

const instance = new AppStateImpl();
export const appState: AppState = instance;
export const { tagStore, articleStore, feedStore, refreshStore, navStore, uiStore, settingsStore, shortcutStore } =
  instance;
