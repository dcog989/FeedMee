import { invoke } from '@tauri-apps/api/core';
import { createArticleActions } from './articleActions.svelte';
import { createFeedActions } from './feedActions.svelte';
import { createFeedRefresher } from './feedRefresh.svelte';
import { createFreshnessHelpers } from './freshness.svelte';
import { registerShortcuts, setupKeyHandler } from './keyboardNav.svelte';
import { createNavigation } from './navigation.svelte';
import { createShortcutOps } from './shortcuts.svelte';
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
} from './storeTypes';
import { createTagOps } from './tags.svelte';
import type { AppSettings, Article, Folder } from './types';
import { DEFAULT_SETTINGS } from './types';
import { createUI } from './ui.svelte';
import {
  LS_BLOCKED_PHRASES,
  LS_LAST_REFRESHED,
  LS_LAST_VIEW_ID,
  LS_LAST_VIEW_TYPE,
  LS_LIST_WIDTH,
  LS_NAV_WIDTH,
  LS_SORT_ORDER,
  LS_THEME,
} from './utils/persistence';

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
} from './storeTypes';
export type { Article, SortOrder, Theme };
export const FEED_ID_LATEST = -1;
export const FEED_ID_SAVED = -2;
export const FEED_ID_TODAY = -3;

export function applyThemeToDocument(theme: Theme) {
  const root = document.documentElement;
  if (theme !== 'system') {
    root.setAttribute('data-theme', theme);
    return;
  }
  const dark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  root.setAttribute('data-theme', dark ? 'dark' : 'light');
}

class AppStateImpl {
  folders = $state<Folder[]>([]);
  articles = $state<Article[]>([]);
  selectedFeedId = $state<number | null>(null);
  selectedFolderId = $state<number | null>(null);
  selectedArticle = $state<Article | null>(null);
  isLoadingArticles = $state(false);
  isRefreshingFeeds = $state(false);
  searchQuery = $state('');
  theme = $state<Theme>('system');
  sortOrder = $state<SortOrder>('desc');

  settings = $state<AppSettings>({ ...DEFAULT_SETTINGS });

  showSettings = $state(false);
  showAddDialog = $state(false);
  showAbout = $state(false);
  showNewFolderDialog = $state(false);
  showEditFeedDialog = $state(false);
  editFeedTarget = $state<{ id: number; name: string; source_type: string; source_id: string } | null>(null);
  renameFolderTarget = $state<{ id: number; name: string } | null>(null);
  expandedFolders = $state<Set<number>>(new Set());
  focusedPane = $state<'nav' | 'list' | 'reading'>('nav');
  blockedPhrases = $state<string[]>([]);
  customShortcuts = $state<Record<string, string>>({});
  navWidth = $state(280);
  listWidth = $state(320);

  page = 0;
  readonly pageSize = 50;
  hasMore = $state(true);
  readonly latestHours = 24;

  lastRefreshed = new Map<number, number>();
  updatingFeedIds = $state(new Set<number>());

  modalState = $state<{
    isOpen: boolean;
    type: 'confirm' | 'alert';
    message: string;
    onConfirm: () => void;
  }>({
    isOpen: false,
    type: 'confirm',
    message: '',
    onConfirm: () => {},
  });

  private refresh: ReturnType<typeof createFeedRefresher>;
  private feedOps: ReturnType<typeof createFeedActions>;
  private articleOps: ReturnType<typeof createArticleActions>;
  private freshness: ReturnType<typeof createFreshnessHelpers>;
  private nav: ReturnType<typeof createNavigation>;
  private tagOps: ReturnType<typeof createTagOps>;
  private shortcutOps: ReturnType<typeof createShortcutOps>;
  private ui: ReturnType<typeof createUI>;
  autoRefreshTimer: ReturnType<typeof setInterval> | null = null;
  constructor() {
    this.refresh = createFeedRefresher(this);
    this.feedOps = createFeedActions(this);
    this.articleOps = createArticleActions(this);
    this.freshness = createFreshnessHelpers(this);
    this.nav = createNavigation(this);
    this.tagOps = createTagOps(this);
    this.shortcutOps = createShortcutOps(this);
    this.ui = createUI(this);
    registerShortcuts(this);
    setupKeyHandler(this);
    this.initStore();
  }

  get debounceMs() {
    return this.settings.feed_refresh_debounce_minutes * 60 * 1000;
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
      const result = await invoke<Folder[]>('get_folders_with_feeds');
      this.folders = result || [];
    } catch (e) {
      console.error('Failed to load folders:', e);
    }
  }

  refreshAllFeeds = () => this.refresh.refreshAllFeeds();
  requestRefreshFeed = (feedId: number) => this.refresh.requestRefreshFeed(feedId);
  requestRefreshFolder = (folderId: number) => this.refresh.requestRefreshFolder(folderId);

  reloadCurrentArticleList = (options?: { selectTop?: boolean }) => this.articleOps.reloadCurrentArticleList(options);
  loadMore = () => this.articleOps.loadMore();
  selectArticle = (article: Article) => this.articleOps.selectArticle(article);
  toggleSaved = (article: Article) => this.articleOps.toggleSaved(article);
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
    const storedNav = localStorage.getItem(LS_NAV_WIDTH);
    const storedList = localStorage.getItem(LS_LIST_WIDTH);
    const storedSort = localStorage.getItem(LS_SORT_ORDER);
    const storedTheme = localStorage.getItem(LS_THEME);
    const storedLastRefreshed = localStorage.getItem(LS_LAST_REFRESHED);

    if (storedNav) this.navWidth = parseInt(storedNav, 10);
    if (storedList) this.listWidth = parseInt(storedList, 10);
    if (storedSort === 'asc' || storedSort === 'desc') this.sortOrder = storedSort;
    if (storedTheme === 'light' || storedTheme === 'dark' || storedTheme === 'system') this.theme = storedTheme;
    applyThemeToDocument(this.theme);

    const storedBlocked = localStorage.getItem(LS_BLOCKED_PHRASES);
    if (storedBlocked) {
      try {
        this.blockedPhrases = JSON.parse(storedBlocked);
      } catch {
        /* ignore */
      }
    }

    if (storedLastRefreshed) {
      try {
        const parsed = JSON.parse(storedLastRefreshed);
        this.lastRefreshed = new Map(Object.entries(parsed).map(([k, v]) => [parseInt(k, 10), v as number]));
      } catch (e) {
        console.error('Failed to parse lastRefreshed', e);
      }
    }

    await Promise.all([
      invoke<AppSettings>('get_app_settings')
        .then((s) => {
          this.settings = s;
          this.ui.startAutoRefreshTimer();
        })
        .catch((e) => console.error('Failed to load settings', e)),
      this.shortcutOps.loadShortcutSettings(),
    ]);

    await this.refreshFolders();

    const viewType = this.settings.default_view_type;
    const viewId = this.settings.default_view_id;

    let expandFolderId: number | null = null;

    if (viewType === 'saved') await this.selectFeed(FEED_ID_SAVED);
    else if (viewType === 'latest') await this.selectFeed(FEED_ID_LATEST);
    else if (viewType === 'last') {
      const lastViewType = localStorage.getItem(LS_LAST_VIEW_TYPE);
      const lastViewId = parseInt(localStorage.getItem(LS_LAST_VIEW_ID) || '0', 10);
      if (lastViewType === 'folder' && lastViewId > 0) {
        await this.selectFolder(lastViewId);
        expandFolderId = lastViewId;
      } else if (lastViewType === 'feed' && lastViewId > 0) {
        await this.selectFeed(lastViewId);
        expandFolderId = this.folders.find((f) => f.feeds.some((fd) => fd.id === lastViewId))?.id ?? null;
      } else {
        await this.selectFeed(FEED_ID_LATEST);
      }
    } else if (viewType === 'folder' && viewId > 0) {
      await this.selectFolder(viewId);
      expandFolderId = viewId;
    } else if (viewType === 'feed' && viewId > 0) {
      await this.selectFeed(viewId);
      expandFolderId = this.folders.find((f) => f.feeds.some((fd) => fd.id === viewId))?.id ?? null;
    }

    {
      const newSet = new Set<number>();
      if (expandFolderId !== null) newSet.add(expandFolderId);
      this.expandedFolders = newSet;
    }

    // Establish the initial view first so the refresh can publish the selected
    // folder's list as soon as its own feeds finish, rather than after the whole
    // batch completes.
    this.refreshAllFeeds();
  }
}

export const appState: AppState = new AppStateImpl();
export const tagStore: TagStore = appState;
export const articleStore: ArticleStore = appState;
export const feedStore: FeedStore = appState;
export const refreshStore: RefreshStore = appState;
export const navStore: NavStore = appState;
export const uiStore: UIStore = appState;
export const settingsStore: SettingsStore = appState;
export const shortcutStore: ShortcutStore = appState;
