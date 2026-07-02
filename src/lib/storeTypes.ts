import type { AppSettings, Article, Folder, Tag } from './types';

export type Theme = 'light' | 'dark' | 'system';
export type SortOrder = 'desc' | 'asc';

export interface TagStore {
  getArticleTags(articleId: number): Promise<Tag[]>;
  getAllTags(): Promise<Tag[]>;
  addTag(articleId: number, name: string, color?: string): Promise<Tag>;
  removeTag(articleId: number, tagId: number): Promise<void>;
  deleteTag(tagId: number): Promise<void>;
}

export interface ArticleStore {
  readonly pageSize: number;
  readonly latestHours: number;
  sortOrder: SortOrder;
  searchQuery: string;
  selectedFeedId: number | null;
  selectedFolderId: number | null;
  blockedPhrases: string[];
  articles: Article[];
  page: number;
  hasMore: boolean;
  isLoadingArticles: boolean;
  focusedPane: 'nav' | 'list' | 'reading';
  selectedArticle: Article | null;
  settings: AppSettings;
  adjustUnreadCount(feedId: number, delta: number): void;
  refreshFolders(): Promise<void>;
  setSearch(query: string): Promise<void>;
  selectArticle(article: Article): void;
  toggleSaved(article: Article): Promise<void>;
  fetchFullContent(article: Article): Promise<string | null>;
  loadMore(): Promise<void>;
}

export interface FeedStore {
  folders: Folder[];
  selectedFeedId: number | null;
  selectedFolderId: number | null;
  articles: Article[];
  isLoadingArticles: boolean;
  lastRefreshed: Map<number, number>;
  blockedPhrases: string[];
  refreshFolders(): Promise<void>;
  refreshAllFeeds(): Promise<void>;
  reloadCurrentArticleList(): Promise<void>;
  alert(message: string): void;
  confirm(message: string, onConfirm: () => void | Promise<void>): void;
  persistLastRefreshed(): void;
  markAllRead(): Promise<void>;
  selectFeed(feedId: number): Promise<void>;
  addFeed(url: string, folderId?: number | null): Promise<void>;
  createFolder(name: string): Promise<void>;
  importOpml(): Promise<void>;
  exportOpml(): Promise<void>;
  renameFolder(id: number, newName: string): Promise<void>;
  renameFeed(id: number, newName: string, newUrl: string): Promise<void>;
  deleteFeed(id: number): Promise<void>;
  deleteFolder(id: number): Promise<void>;
  moveFeed(feedId: number, folderId: number | null): Promise<void>;
  setBlockedPhrases(phrases: string[]): Promise<void>;
}

export interface RefreshStore {
  folders: Folder[];
  selectedFeedId: number | null;
  selectedFolderId: number | null;
  lastRefreshed: Map<number, number>;
  updatingFeedIds: Set<number>;
  isRefreshingFeeds: boolean;
  readonly debounceMs: number;
  persistLastRefreshed(): void;
  isFeedFresh(feedId: number): boolean;
  isFeedUpdating(feedId: number): boolean;
  isFolderUpdating(folderId: number): boolean;
  isFolderFresh(folderId: number): boolean;
  isAllFresh(): boolean;
  refreshFolders(): Promise<void>;
  reloadCurrentArticleList(): Promise<void>;
  refreshAllFeeds(): Promise<void>;
  requestRefreshFeed(feedId: number): Promise<void>;
  requestRefreshFolder(folderId: number): Promise<void>;
}

export interface NavStore {
  folders: Folder[];
  expandedFolders: Set<number>;
  selectedFeedId: number | null;
  selectedFolderId: number | null;
  selectedArticle: Article | null;
  articles: Article[];
  searchQuery: string;
  isLoadingArticles: boolean;
  settings: AppSettings;
  focusedPane: 'nav' | 'list' | 'reading';
  reloadCurrentArticleList(): Promise<void>;
  selectFeed(feedId: number): Promise<void>;
  selectFolder(folderId: number): Promise<void>;
  selectArticle(article: Article): void;
  navUp(): void;
  navDown(): void;
  articleUp(): void;
  articleDown(): void;
}

export interface UIStore {
  showSettings: boolean;
  showAddDialog: boolean;
  showAbout: boolean;
  showNewFolderDialog: boolean;
  showEditFeedDialog: boolean;
  editFeedTarget: { id: number; name: string; url: string } | null;
  focusedPane: 'nav' | 'list' | 'reading';
  modalState: {
    isOpen: boolean;
    type: 'confirm' | 'alert';
    message: string;
    onConfirm: () => void;
  };
  openSettings(): void;
  closeSettings(): void;
  openAbout(): void;
  closeAbout(): void;
  confirm(message: string, onConfirm: () => void | Promise<void>): void;
  alert(message: string): void;
  closeModal(): void;
}

export interface SettingsStore {
  theme: Theme;
  sortOrder: SortOrder;
  settings: AppSettings;
  showSettings: boolean;
  focusedPane: 'nav' | 'list' | 'reading';
  openSettings(): void;
  closeSettings(): void;
  saveSettings(newSettings: AppSettings, closeModal?: boolean): Promise<void>;
  setTheme(theme: Theme): void;
  setSortOrder(order: SortOrder): Promise<void>;
  persistLayoutSettings(): void;
}

export interface ShortcutStore {
  customShortcuts: Record<string, string>;
  setShortcut(commandId: string, key: string): void;
  resetShortcut(commandId: string): void;
}

export interface AppState {
  folders: Folder[];
  articles: Article[];
  selectedFeedId: number | null;
  selectedFolderId: number | null;
  selectedArticle: Article | null;
  isLoadingArticles: boolean;
  isRefreshingFeeds: boolean;
  searchQuery: string;
  theme: Theme;
  sortOrder: SortOrder;
  settings: AppSettings;
  showSettings: boolean;
  showAddDialog: boolean;
  showAbout: boolean;
  showNewFolderDialog: boolean;
  showEditFeedDialog: boolean;
  editFeedTarget: { id: number; name: string; url: string } | null;
  expandedFolders: Set<number>;
  blockedPhrases: string[];
  focusedPane: 'nav' | 'list' | 'reading';
  customShortcuts: Record<string, string>;
  navWidth: number;
  listWidth: number;
  page: number;
  readonly pageSize: number;
  hasMore: boolean;
  latestHours: number;
  lastRefreshed: Map<number, number>;
  updatingFeedIds: Set<number>;
  modalState: {
    isOpen: boolean;
    type: 'confirm' | 'alert';
    message: string;
    onConfirm: () => void;
  };
  readonly debounceMs: number;
  persistLastRefreshed(): void;
  isFeedFresh(feedId: number): boolean;
  isFolderFresh(folderId: number): boolean;
  isAllFresh(): boolean;
  adjustUnreadCount(feedId: number, delta: number): void;
  isFeedUpdating(feedId: number): boolean;
  isFolderUpdating(folderId: number): boolean;
  refreshFolders(): Promise<void>;
  reloadCurrentArticleList(): Promise<void>;
  refreshAllFeeds(): Promise<void>;
  requestRefreshFeed(feedId: number): Promise<void>;
  requestRefreshFolder(folderId: number): Promise<void>;
  selectFeed(feedId: number): Promise<void>;
  selectFolder(folderId: number): Promise<void>;
  selectArticle(article: Article): void;
  toggleSaved(article: Article): Promise<void>;
  fetchFullContent(article: Article): Promise<string | null>;
  loadMore(): Promise<void>;
  markAllRead(): Promise<void>;
  addFeed(url: string, folderId?: number | null): Promise<void>;
  createFolder(name: string): Promise<void>;
  importOpml(): Promise<void>;
  exportOpml(): Promise<void>;
  renameFolder(id: number, newName: string): Promise<void>;
  renameFeed(id: number, newName: string, newUrl: string): Promise<void>;
  deleteFeed(id: number): Promise<void>;
  deleteFolder(id: number): Promise<void>;
  moveFeed(feedId: number, folderId: number | null): Promise<void>;
  navUp(): void;
  navDown(): void;
  articleUp(): void;
  articleDown(): void;
  setShortcut(commandId: string, key: string): void;
  resetShortcut(commandId: string): void;
  persistLayoutSettings(): void;
  setBlockedPhrases(phrases: string[]): Promise<void>;
  setSortOrder(order: SortOrder): Promise<void>;
  setSearch(query: string): Promise<void>;
  setTheme(theme: Theme): void;
  openSettings(): void;
  closeSettings(): void;
  openAbout(): void;
  closeAbout(): void;
  saveSettings(newSettings: AppSettings, closeModal?: boolean): Promise<void>;
  confirm(message: string, onConfirm: () => void | Promise<void>): void;
  alert(message: string): void;
  closeModal(): void;
  getArticleTags(articleId: number): Promise<Tag[]>;
  getAllTags(): Promise<Tag[]>;
  addTag(articleId: number, name: string, color?: string): Promise<Tag>;
  removeTag(articleId: number, tagId: number): Promise<void>;
  deleteTag(tagId: number): Promise<void>;
}
