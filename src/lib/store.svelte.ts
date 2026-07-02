import { invoke } from '@tauri-apps/api/core';
import { createArticleActions } from './articleActions.svelte';
import { createFeedActions } from './feedActions.svelte';
import { createFeedRefresher } from './feedRefresh.svelte';
import { createFreshnessHelpers } from './freshness.svelte';
import { registerShortcuts, setupKeyHandler } from './keyboardNav.svelte';
import { createNavigation } from './navigation.svelte';
import { createShortcutOps } from './shortcuts.svelte';
import type { AppState, SortOrder, Theme } from './storeTypes';
import { createTagOps } from './tags.svelte';
import type { AppSettings, Article, Folder, Tag } from './types';

export type { AppState } from './storeTypes';
export type { Article, SortOrder, Theme };
export const FEED_ID_LATEST = -1;
export const FEED_ID_SAVED = -2;
export const FEED_ID_TODAY = -3;

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

  settings = $state<AppSettings>({
    feed_refresh_debounce_minutes: 4,
    auto_update_interval_minutes: 30,
    log_level: 'info',
    default_view_type: 'latest',
    default_view_id: -1,
    auto_collapse_folders: true,
    mark_feed_read_on_exit: false,
    article_title_font: '',
    article_body_font: '',
    article_title_color: '',
    article_body_color: '',
    article_bg_color: '',
    thumbnail_size: 0,
    article_retention_days: 90,
  });

  showSettings = $state(false);
  showAddDialog = $state(false);
  showAbout = $state(false);
  showNewFolderDialog = $state(false);
  showEditFeedDialog = $state(false);
  editFeedTarget = $state<{ id: number; name: string; url: string } | null>(null);
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
  private autoRefreshTimer: ReturnType<typeof setInterval> | null = null;
  private cleanupKeyHandler: (() => void) | null = null;

  constructor() {
    this.refresh = createFeedRefresher(this);
    this.feedOps = createFeedActions(this);
    this.articleOps = createArticleActions(this);
    this.freshness = createFreshnessHelpers(this);
    this.nav = createNavigation(this);
    this.tagOps = createTagOps(this);
    this.shortcutOps = createShortcutOps(this);
    registerShortcuts(this);
    this.cleanupKeyHandler = setupKeyHandler(this);
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

  reloadCurrentArticleList = () => this.articleOps.reloadCurrentArticleList();
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

  async setBlockedPhrases(phrases: string[]) {
    this.blockedPhrases = phrases;
    localStorage.setItem('blockedPhrases', JSON.stringify(phrases));
    await this.reloadCurrentArticleList();
  }

  persistLayoutSettings() {
    localStorage.setItem('navWidth', this.navWidth.toString());
    localStorage.setItem('listWidth', this.listWidth.toString());
    localStorage.setItem('sortOrder', this.sortOrder);
  }

  async setSortOrder(order: SortOrder) {
    if (this.sortOrder !== order) {
      this.sortOrder = order;
      this.persistLayoutSettings();
      await this.reloadCurrentArticleList();
    }
  }

  async setSearch(query: string) {
    this.searchQuery = query;
    await this.reloadCurrentArticleList();
  }

  setTheme(newTheme: Theme) {
    this.theme = newTheme;
    localStorage.setItem('theme', newTheme);
  }

  openSettings() {
    this.showSettings = true;
  }

  closeSettings() {
    this.showSettings = false;
  }

  openAbout() {
    this.showAbout = true;
  }

  closeAbout() {
    this.showAbout = false;
  }

  async saveSettings(newSettings: AppSettings, closeModal = true) {
    try {
      await invoke('save_app_settings', { newSettings });
      this.settings = newSettings;
      if (this.autoRefreshTimer !== null) {
        clearInterval(this.autoRefreshTimer);
        this.autoRefreshTimer = null;
      }
      this.startAutoRefreshTimer();
      if (closeModal) this.closeSettings();
    } catch (e) {
      this.alert(`Failed to save settings: ${e}`);
    }
  }

  confirm(message: string, onConfirm: () => void | Promise<void>) {
    this.modalState = {
      isOpen: true,
      type: 'confirm',
      message,
      onConfirm: () => {
        this.modalState.isOpen = false;
        Promise.resolve(onConfirm()).catch((e) => console.error('confirm callback failed:', e));
      },
    };
  }

  alert(message: string) {
    this.modalState = {
      isOpen: true,
      type: 'alert',
      message,
      onConfirm: () => {
        this.modalState.isOpen = false;
      },
    };
  }

  closeModal() {
    this.modalState.isOpen = false;
  }

  private startAutoRefreshTimer() {
    if (this.settings.auto_update_interval_minutes > 0) {
      const intervalMs = this.settings.auto_update_interval_minutes * 60 * 1000;
      this.autoRefreshTimer = setInterval(() => this.refreshAllFeeds(), intervalMs);
    }
  }

  private async initStore() {
    const storedNav = localStorage.getItem('navWidth');
    const storedList = localStorage.getItem('listWidth');
    const storedSort = localStorage.getItem('sortOrder');
    const storedTheme = localStorage.getItem('theme');
    const storedLastRefreshed = localStorage.getItem('lastRefreshed');

    if (storedNav) this.navWidth = parseInt(storedNav, 10);
    if (storedList) this.listWidth = parseInt(storedList, 10);
    if (storedSort === 'asc' || storedSort === 'desc') this.sortOrder = storedSort;
    if (storedTheme === 'light' || storedTheme === 'dark' || storedTheme === 'system') this.theme = storedTheme;

    const storedBlocked = localStorage.getItem('blockedPhrases');
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

    try {
      const s = await invoke<AppSettings>('get_app_settings');
      this.settings = s;
      this.startAutoRefreshTimer();
    } catch (e) {
      console.error('Failed to load settings', e);
    }

    await this.shortcutOps.loadShortcutSettings();
    await this.refreshFolders();
    this.refreshAllFeeds();

    const viewType = this.settings.default_view_type;
    const viewId = this.settings.default_view_id;

    if (viewType === 'saved') this.selectFeed(FEED_ID_SAVED);
    else if (viewType === 'latest') this.selectFeed(FEED_ID_LATEST);
    else if (viewType === 'last') {
      const lastViewType = localStorage.getItem('lastViewType');
      const lastViewId = parseInt(localStorage.getItem('lastViewId') || '0', 10);
      if (lastViewType === 'folder' && lastViewId > 0) {
        this.selectFolder(lastViewId);
      } else if (lastViewType === 'feed' && lastViewId > 0) {
        this.selectFeed(lastViewId);
      } else {
        this.selectFeed(FEED_ID_LATEST);
      }
    } else if (viewType === 'folder' && viewId > 0) this.selectFolder(viewId);
    else if (viewType === 'feed' && viewId > 0) this.selectFeed(viewId);
  }
}

export const appState: AppState = new AppStateImpl();
