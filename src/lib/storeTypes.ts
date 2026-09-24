import type { AppSettings, Article, Folder, Tag } from "./types";

export type Theme = "light" | "dark" | "system";
export type SortOrder = "desc" | "asc";

export interface TagStore {
  getArticleTags(articleId: number): Promise<Tag[]>;
  getAllTags(): Promise<Tag[]>;
  addTag(articleId: number, name: string, color?: string): Promise<Tag>;
  removeTag(articleId: number, tagId: number): Promise<void>;
  deleteTag(tagId: number): Promise<void>;
}

export interface FeedSelection {
  selectedFeedId: number | null;
  selectedFolderId: number | null;
}

export interface ArticleSelection {
  selectedArticle: Article | null;
  selectArticle(article: Article): void;
}

export interface FolderCollection {
  folders: Folder[];
}

export interface ArticleCollection {
  articles: Article[];
  isLoadingArticles: boolean;
}

export interface SearchState {
  searchQuery: string;
}

export interface SortState {
  sortOrder: SortOrder;
}

export interface AppSettingsRef {
  settings: AppSettings;
}

export interface PaneFocus {
  focusedPane: "nav" | "list" | "reading";
}

export interface BlockedPhrasesState {
  blockedPhrases: string[];
}

export interface RefreshTimestampState {
  lastRefreshed: Map<number, number>;
  persistLastRefreshed(): void;
}

export interface RefreshFolders {
  refreshFolders(): Promise<void>;
}

export interface RefreshAllFeeds extends RefreshFolders {
  refreshAllFeeds(): Promise<void>;
}

export interface ReloadArticleList {
  reloadCurrentArticleList(options?: { selectTop?: boolean }): Promise<void>;
}

export interface SelectFeed {
  selectFeed(feedId: number): Promise<void>;
}

export interface SettingsDialog {
  showSettings: boolean;
  openSettings(): void;
  closeSettings(): void;
}

export interface UserPrompt {
  alert(message: string): void;
  confirm(message: string, onConfirm: () => void | Promise<void>): void;
}

export interface ArticleStore
  extends FeedSelection,
    ArticleSelection,
    ArticleCollection,
    SearchState,
    SortState,
    AppSettingsRef,
    PaneFocus,
    BlockedPhrasesState,
    RefreshFolders {
  readonly pageSize: number;
  readonly latestHours: number;
  page: number;
  hasMore: boolean;
  adjustUnreadCount(feedId: number, delta: number): void;
  setSearch(query: string): Promise<void>;
  toggleSaved(article: Article): Promise<void>;
  fetchFullContent(article: Article): Promise<string | null>;
  loadMore(): Promise<void>;
}

export interface FeedStore
  extends FeedSelection,
    FolderCollection,
    ArticleCollection,
    RefreshTimestampState,
    BlockedPhrasesState,
    RefreshAllFeeds,
    ReloadArticleList,
    SelectFeed,
    UserPrompt {
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
  setBlockedPhrases(phrases: string[]): Promise<void>;
}

export interface NavStore
  extends FeedSelection,
    FolderCollection,
    ArticleSelection,
    ArticleCollection,
    SearchState,
    AppSettingsRef,
    PaneFocus,
    ReloadArticleList,
    SelectFeed {
  expandedFolders: Set<number>;
  selectFolder(folderId: number): Promise<void>;
  navUp(): void;
  navDown(): void;
  articleUp(): void;
  articleDown(): void;
}

export interface RefreshStore
  extends FeedSelection,
    FolderCollection,
    RefreshTimestampState,
    RefreshAllFeeds,
    ReloadArticleList {
  updatingFeedIds: Set<number>;
  isRefreshingFeeds: boolean;
  readonly debounceMs: number;
  isFeedFresh(feedId: number): boolean;
  isFeedUpdating(feedId: number): boolean;
  isFolderUpdating(folderId: number): boolean;
  isFolderFresh(folderId: number): boolean;
  isAllFresh(): boolean;
  requestRefreshFeed(feedId: number): Promise<void>;
  requestRefreshFolder(folderId: number): Promise<void>;
}

export interface UIStore extends PaneFocus, SettingsDialog, UserPrompt {
  showAddDialog: boolean;
  showAbout: boolean;
  showNewFolderDialog: boolean;
  showEditFeedDialog: boolean;
  editFeedTarget: { id: number; name: string; source_type: string; source_id: string } | null;
  renameFolderTarget: { id: number; name: string } | null;
  modalState: {
    isOpen: boolean;
    type: "confirm" | "alert";
    message: string;
    onConfirm: () => void;
  };
  openAbout(): void;
  closeAbout(): void;
  closeModal(): void;
}

export interface SettingsStore extends SortState, AppSettingsRef, PaneFocus, SettingsDialog {
  theme: Theme;
  navWidth: number;
  listWidth: number;
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

export interface AppState
  extends ArticleStore,
    FeedStore,
    NavStore,
    UIStore,
    SettingsStore,
    ShortcutStore,
    TagStore,
    RefreshStore {}
