import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, Article, Folder } from './types';
import { shortcutManager } from './utils/shortcuts';
import { createFeedRefresher } from './feedRefresh.svelte';
import { createFeedActions } from './feedActions.svelte';
import { createArticleActions } from './articleActions.svelte';
import { registerShortcuts, setupKeyHandler } from './keyboardNav.svelte';
import type { AppState, Theme, SortOrder } from './storeTypes';

export type { AppState } from './storeTypes';
export type { Theme, SortOrder };
export type { Article };
export const FEED_ID_LATEST = -1;
export const FEED_ID_SAVED = -2;

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
        refresh_all_debounce_minutes: 0,
        auto_update_interval_minutes: 30,
        log_level: 'info',
        default_view_type: 'latest',
        default_view_id: -1,
        auto_collapse_folders: true,
        mark_feed_read_on_exit: false,
    });

    showSettings = $state(false);
    showAddDialog = $state(false);
    expandedFolders = $state<Set<number>>(new Set());
    focusedPane = $state<'nav' | 'list' | 'reading'>('nav');
    customShortcuts = $state<Record<string, string>>({});
    navWidth = $state(280);
    listWidth = $state(320);

    page = 0;
    readonly pageSize = 50;
    hasMore = $state(true);
    latestHours = $state(24);

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

    constructor() {
        this.refresh = createFeedRefresher(this);
        this.feedOps = createFeedActions(this);
        this.articleOps = createArticleActions(this);
        registerShortcuts(this);
        setupKeyHandler(this);
        this.initStore();
    }

    get debounceMs() {
        return this.settings.feed_refresh_debounce_minutes * 60 * 1000;
    }

    isFeedUpdating(feedId: number) {
        return this.updatingFeedIds.has(feedId);
    }

    isFolderUpdating(folderId: number) {
        const folder = this.folders.find((f) => f.id === folderId);
        if (!folder) return false;
        return folder.feeds.some((feed) => this.updatingFeedIds.has(feed.id));
    }

    isFeedFresh(feedId: number): boolean {
        return Date.now() - (this.lastRefreshed.get(feedId) || 0) < this.debounceMs;
    }

    isFolderFresh(folderId: number): boolean {
        const folder = this.folders.find((f) => f.id === folderId);
        if (!folder || folder.feeds.length === 0) return false;
        return folder.feeds.every((f) => this.isFeedFresh(f.id));
    }

    isAllFresh(): boolean {
        return this.folders.flatMap((f) => f.feeds).every((f) => this.isFeedFresh(f.id));
    }

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
    renameFeed = (id: number, newName: string) => this.feedOps.renameFeed(id, newName);
    deleteFeed = (id: number) => this.feedOps.deleteFeed(id);
    deleteFolder = (id: number) => this.feedOps.deleteFolder(id);
    moveFeed = (feedId: number, folderId: number) => this.feedOps.moveFeed(feedId, folderId);

    async setSortOrder(order: SortOrder) {
        if (this.sortOrder !== order) {
            this.sortOrder = order;
            await this.reloadCurrentArticleList();
        }
    }

    async setSearch(query: string) {
        this.searchQuery = query;
        await this.reloadCurrentArticleList();
    }

    private async markFeedReadOnExit(previousFeedId: number | null) {
        if (!this.settings.mark_feed_read_on_exit || !previousFeedId || previousFeedId <= 0) return;
        try {
            await invoke('mark_all_read', { targetType: 'feed', id: previousFeedId });
            const unreadCount = await invoke<number>('get_feed_unread_count', {
                feedId: previousFeedId,
            });
            for (const folder of this.folders) {
                const feed = folder.feeds.find((f) => f.id === previousFeedId);
                if (feed) {
                    feed.unread_count = unreadCount;
                    break;
                }
            }
        } catch (e) {
            console.error('mark_feed_read_on_exit failed:', e);
        }
    }

    async selectFolder(folderId: number) {
        if (this.selectedFolderId === folderId && !this.selectedFeedId) return;
        await this.markFeedReadOnExit(this.selectedFeedId);
        this.focusedPane = 'nav';
        this.searchQuery = '';
        this.selectedFolderId = folderId;
        this.selectedFeedId = null;
        this.selectedArticle = null;
        this.isLoadingArticles = true;
        try {
            await this.reloadCurrentArticleList();
        } finally {
            this.isLoadingArticles = false;
        }
    }

    async selectFeed(feedId: number) {
        if (this.selectedFeedId === feedId) return;
        await this.markFeedReadOnExit(this.selectedFeedId);
        this.focusedPane = 'nav';
        this.searchQuery = '';
        this.selectedFeedId = feedId;
        this.selectedFolderId = null;
        this.selectedArticle = null;
        this.isLoadingArticles = true;
        try {
            await this.reloadCurrentArticleList();
        } finally {
            this.isLoadingArticles = false;
        }
    }

    private getFlatNavItems(): { type: 'feed' | 'folder'; id: number }[] {
        const items: { type: 'feed' | 'folder'; id: number }[] = [];
        for (const folder of this.folders) {
            items.push({ type: 'folder', id: folder.id });
            if (this.expandedFolders.has(folder.id)) {
                for (const feed of folder.feeds) {
                    items.push({ type: 'feed', id: feed.id });
                }
            }
        }
        return items;
    }

    private expandFolder(folderId: number) {
        const newSet = new Set(this.expandedFolders);
        if (this.settings.auto_collapse_folders) newSet.clear();
        newSet.add(folderId);
        this.expandedFolders = newSet;
    }

    navUp() {
        const items = this.getFlatNavItems();
        if (items.length === 0) return;
        const currentIdx = items.findIndex(
            (i) =>
                (i.type === 'feed' && i.id === this.selectedFeedId) ||
                (i.type === 'folder' && i.id === this.selectedFolderId && !this.selectedFeedId),
        );
        const nextIdx = currentIdx <= 0 ? items.length - 1 : currentIdx - 1;
        const item = items[nextIdx];
        if (item.type === 'feed') this.selectFeed(item.id);
        else {
            this.expandFolder(item.id);
            this.selectFolder(item.id);
        }
    }

    navDown() {
        const items = this.getFlatNavItems();
        if (items.length === 0) return;
        const currentIdx = items.findIndex(
            (i) =>
                (i.type === 'feed' && i.id === this.selectedFeedId) ||
                (i.type === 'folder' && i.id === this.selectedFolderId && !this.selectedFeedId),
        );
        const nextIdx = currentIdx < 0 || currentIdx >= items.length - 1 ? 0 : currentIdx + 1;
        const item = items[nextIdx];
        if (item.type === 'feed') this.selectFeed(item.id);
        else {
            this.expandFolder(item.id);
            this.selectFolder(item.id);
        }
    }

    articleUp() {
        if (this.articles.length === 0) return;
        const idx = this.articles.findIndex((a) => a.id === this.selectedArticle?.id);
        const nextIdx = idx <= 0 ? 0 : idx - 1;
        this.selectArticle(this.articles[nextIdx]);
        this.scrollSelectedIntoView('.list-area .article-card.selected');
    }

    articleDown() {
        if (this.articles.length === 0) return;
        const idx = this.articles.findIndex((a) => a.id === this.selectedArticle?.id);
        const nextIdx = idx < 0 ? 0 : Math.min(idx + 1, this.articles.length - 1);
        this.selectArticle(this.articles[nextIdx]);
        this.scrollSelectedIntoView('.list-area .article-card.selected');
    }

    private scrollSelectedIntoView(selector: string) {
        requestAnimationFrame(() => {
            document.querySelector<HTMLElement>(selector)?.scrollIntoView({ block: 'nearest' });
        });
    }

    setTheme(newTheme: Theme) {
        this.theme = newTheme;
    }
    openSettings() {
        this.showSettings = true;
    }
    closeSettings() {
        this.showSettings = false;
    }

    async saveSettings(newSettings: AppSettings, closeModal = true) {
        try {
            await invoke('save_app_settings', { newSettings });
            this.settings = newSettings;
            if (closeModal) this.closeSettings();
        } catch (e) {
            this.alert(`Failed to save settings: ${e}`);
        }
    }

    confirm(message: string, onConfirm: () => void) {
        this.modalState = {
            isOpen: true,
            type: 'confirm',
            message,
            onConfirm: () => {
                onConfirm();
                this.modalState.isOpen = false;
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

    setShortcut(commandId: string, key: string) {
        this.customShortcuts[commandId] = key;
        shortcutManager.setCustomMappings(this.customShortcuts);
        this.saveShortcutSettings();
    }

    resetShortcut(commandId: string) {
        delete this.customShortcuts[commandId];
        shortcutManager.setCustomMappings(this.customShortcuts);
        this.saveShortcutSettings();
    }

    private async saveShortcutSettings() {
        try {
            await invoke('save_shortcuts', { shortcuts: this.customShortcuts });
        } catch (e) {
            console.error('Failed to save shortcuts:', e);
        }
    }

    private async loadShortcutSettings() {
        try {
            const shortcuts = await invoke<Record<string, string>>('get_shortcuts');
            this.customShortcuts = shortcuts || {};
            shortcutManager.setCustomMappings(this.customShortcuts);
        } catch (e) {
            console.error('Failed to load shortcuts:', e);
        }
    }

    private async initStore() {
        const storedNav = localStorage.getItem('navWidth');
        const storedList = localStorage.getItem('listWidth');
        const storedSort = localStorage.getItem('sortOrder');
        const storedLastRefreshed = localStorage.getItem('lastRefreshed');

        if (storedNav) this.navWidth = parseInt(storedNav);
        if (storedList) this.listWidth = parseInt(storedList);
        if (storedSort === 'asc' || storedSort === 'desc') this.sortOrder = storedSort;

        if (storedLastRefreshed) {
            try {
                const parsed = JSON.parse(storedLastRefreshed);
                this.lastRefreshed = new Map(
                    Object.entries(parsed).map(([k, v]) => [parseInt(k), v as number]),
                );
            } catch (e) {
                console.error('Failed to parse lastRefreshed', e);
            }
        }

        try {
            const s = await invoke<AppSettings>('get_app_settings');
            this.settings = s;
            if (this.settings.auto_update_interval_minutes > 0) {
                const intervalMs = this.settings.auto_update_interval_minutes * 60 * 1000;
                setInterval(() => this.refreshAllFeeds(), intervalMs);
            }
        } catch (e) {
            console.error('Failed to load settings', e);
        }

        await this.loadShortcutSettings();
        await this.refreshFolders();
        this.refreshAllFeeds();

        const viewType = this.settings.default_view_type;
        const viewId = this.settings.default_view_id;

        if (viewType === 'saved') this.selectFeed(FEED_ID_SAVED);
        else if (viewType === 'latest') this.selectFeed(FEED_ID_LATEST);
        else if (viewType === 'folder' && viewId > 0) this.selectFolder(viewId);
        else if (viewType === 'feed' && viewId > 0) this.selectFeed(viewId);

        $effect.root(() => {
            $effect(() => {
                localStorage.setItem('navWidth', this.navWidth.toString());
                localStorage.setItem('listWidth', this.listWidth.toString());
                localStorage.setItem('sortOrder', this.sortOrder);
            });
        });
    }
}

export const appState: AppState = new AppStateImpl();
