import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import type { AppSettings, Article, Feed, Folder } from './types';
import { shortcutManager } from './utils/shortcuts';

export type Theme = 'light' | 'dark' | 'sepia' | 'system';
export type SortOrder = 'desc' | 'asc';

export const FEED_ID_LATEST = -1;
export const FEED_ID_SAVED = -2;

const REFRESH_CONCURRENCY = 5;

class AppState {
    folders = $state<Folder[]>([]);
    articles = $state<Article[]>([]);
    selectedFeedId = $state<number | null>(null);
    selectedFolderId = $state<number | null>(null);
    selectedArticle = $state<Article | null>(null);
    isLoading = $state(false);
    searchQuery = $state('');
    theme = $state<Theme>('system');
    sortOrder = $state<SortOrder>('desc');

    settings = $state<AppSettings>({
        feed_refresh_debounce_minutes: 4,
        refresh_all_debounce_minutes: 0,
        auto_update_interval_minutes: 30,
        log_level: 'info',
    });

    // UI States
    showSettings = $state(false);
    showAddDialog = $state(false);
    expandedFolders = $state<Set<number>>(new Set());

    // Custom keyboard shortcuts
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

    constructor() {
        this.initStore();
        this.registerShortcuts();
        this.setupKeyHandler();
    }

    private registerShortcuts() {
        shortcutManager.register({
            id: 'settings',
            command: 'settings',
            defaultKey: ',',
            description: 'Open settings',
            category: 'General',
            handler: () => this.openSettings(),
        });

        shortcutManager.register({
            id: 'add-feed',
            command: 'add-feed',
            defaultKey: 'n',
            description: 'Add new feed',
            category: 'General',
            handler: () => {
                this.showAddDialog = true;
            },
        });

        shortcutManager.register({
            id: 'refresh-all',
            command: 'refresh-all',
            defaultKey: 'r',
            description: 'Refresh all feeds',
            category: 'Feeds',
            handler: () => this.refreshAllFeeds(),
        });

        shortcutManager.register({
            id: 'focus-search',
            command: 'focus-search',
            defaultKey: '/',
            description: 'Focus search',
            category: 'General',
            handler: () => {
                const searchInput = document.querySelector(
                    '.search-wrapper input',
                ) as HTMLInputElement;
                searchInput?.focus();
            },
        });

        shortcutManager.register({
            id: 'toggle-save',
            command: 'toggle-save',
            defaultKey: 's',
            description: 'Save/Read later',
            category: 'Articles',
            handler: () => {
                if (this.selectedArticle) {
                    this.toggleSaved(this.selectedArticle);
                }
            },
        });

        shortcutManager.register({
            id: 'mark-read',
            command: 'mark-read',
            defaultKey: 'm',
            description: 'Mark as read/unread',
            category: 'Articles',
            handler: async () => {
                if (this.selectedArticle) {
                    const article = this.selectedArticle;
                    const newReadState = !article.is_read;
                    article.is_read = newReadState;
                    await invoke('mark_article_read', { id: article.id, read: newReadState });
                }
            },
        });

        shortcutManager.register({
            id: 'focus-search',
            command: 'focus-search',
            defaultKey: '/',
            description: 'Focus search',
            category: 'General',
            handler: () => {
                const searchInput = document.querySelector(
                    '.search-wrapper input',
                ) as HTMLInputElement;
                searchInput?.focus();
            },
        });

        shortcutManager.register({
            id: 'expand-all',
            command: 'expand-all',
            defaultKey: 'x',
            description: 'Expand all folders',
            category: 'Feeds',
            handler: () => {
                const newSet = new Set<number>();
                this.folders.forEach((f) => newSet.add(f.id));
                this.expandedFolders = newSet;
            },
        });

        shortcutManager.register({
            id: 'collapse-all',
            command: 'collapse-all',
            defaultKey: 'c',
            description: 'Collapse all folders',
            category: 'Feeds',
            handler: () => {
                this.expandedFolders = new Set<number>();
            },
        });

        shortcutManager.register({
            id: 'open-article',
            command: 'open-article',
            defaultKey: 'enter',
            description: 'Open article in browser',
            category: 'Articles',
            handler: () => {
                if (this.selectedArticle) {
                    import('@tauri-apps/plugin-opener').then(({ openUrl }) => {
                        openUrl(this.selectedArticle!.url);
                    });
                }
            },
        });
    }

    private setupKeyHandler() {
        window.addEventListener('keydown', (e) => {
            if (this.showSettings) return;
            shortcutManager.handleKeyEvent(e);
        });
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

        if (storedNav) this.navWidth = parseInt(storedNav);
        if (storedList) this.listWidth = parseInt(storedList);
        if (storedSort === 'asc' || storedSort === 'desc') this.sortOrder = storedSort;

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

        $effect.root(() => {
            $effect(() => {
                localStorage.setItem('navWidth', this.navWidth.toString());
                localStorage.setItem('listWidth', this.listWidth.toString());
                localStorage.setItem('sortOrder', this.sortOrder);
            });
        });
    }

    async refreshFolders() {
        try {
            const result = await invoke<Folder[]>('get_folders_with_feeds');
            this.folders = result || [];
        } catch (e) {
            console.error('Failed to load folders:', e);
        }
    }

    isFeedUpdating(feedId: number) {
        return this.updatingFeedIds.has(feedId);
    }

    isFolderUpdating(folderId: number) {
        const folder = this.folders.find((f) => f.id === folderId);
        if (!folder) return false;
        return folder.feeds.some((feed) => this.updatingFeedIds.has(feed.id));
    }

    get debounceMs() {
        return this.settings.feed_refresh_debounce_minutes * 60 * 1000;
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

    async refreshAllFeeds() {
        const staleFeeds: Feed[] = this.folders
            .flatMap((f) => f.feeds)
            .filter((f) => !this.isFeedFresh(f.id));
        if (staleFeeds.length === 0) return;

        this.isLoading = true;

        const newSet = new Set(this.updatingFeedIds);
        staleFeeds.forEach((f) => newSet.add(f.id));
        this.updatingFeedIds = newSet;

        let index = 0;
        const worker = async () => {
            while (index < staleFeeds.length) {
                const feed = staleFeeds[index++];
                await this.performSingleFeedRefresh(feed.id);
            }
        };

        const workers = Array(REFRESH_CONCURRENCY)
            .fill(null)
            .map(() => worker());

        try {
            await Promise.all(workers);
            await this.refreshFolders();

            if (this.selectedFeedId) {
                await this.reloadCurrentArticleList();
            } else if (this.selectedFolderId) {
                await this.reloadCurrentArticleList();
            }
        } catch (e) {
            console.error('Failed to refresh all feeds:', e);
        } finally {
            this.updatingFeedIds = new Set();
            this.isLoading = false;
        }
    }

    async requestRefreshFeed(feedId: number) {
        if (this.isFeedFresh(feedId)) return;

        const newSet = new Set(this.updatingFeedIds);
        newSet.add(feedId);
        this.updatingFeedIds = newSet;

        try {
            await this.performSingleFeedRefresh(feedId);
        } finally {
            const endSet = new Set(this.updatingFeedIds);
            endSet.delete(feedId);
            this.updatingFeedIds = endSet;
        }

        if (this.selectedFeedId === feedId) {
            await this.reloadCurrentArticleList();
        }
    }

    async requestRefreshFolder(folderId: number) {
        const folder = this.folders.find((f) => f.id === folderId);
        if (!folder || folder.feeds.length === 0) return;

        const newSet = new Set(this.updatingFeedIds);
        folder.feeds.forEach((f) => newSet.add(f.id));
        this.updatingFeedIds = newSet;

        let index = 0;
        const worker = async () => {
            while (index < folder.feeds.length) {
                const feed = folder.feeds[index++];
                await this.performSingleFeedRefresh(feed.id);
            }
        };

        const workers = Array(REFRESH_CONCURRENCY)
            .fill(null)
            .map(() => worker());

        try {
            await Promise.all(workers);
            await this.refreshFolders();
            if (
                this.selectedFolderId === folderId ||
                folder.feeds.some((f) => f.id === this.selectedFeedId)
            ) {
                await this.reloadCurrentArticleList();
            }
        } catch (e) {
            console.error(`Failed to refresh folder ${folderId}:`, e);
        }
    }

    private async performSingleFeedRefresh(feedId: number) {
        try {
            await invoke('refresh_feed', { feedId });
            this.lastRefreshed.set(feedId, Date.now());
        } catch (e) {
            console.error(`Failed to refresh feed ${feedId}:`, e);
        } finally {
            const newSet = new Set(this.updatingFeedIds);
            newSet.delete(feedId);
            this.updatingFeedIds = newSet;
        }
    }

    async reloadCurrentArticleList() {
        this.articles = [];
        this.page = 0;
        const result = await this.fetchPage(0);
        this.articles = result || [];
        this.hasMore = (result?.length || 0) === this.pageSize;
    }

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

    async markAllRead() {
        try {
            if (this.selectedFeedId && this.selectedFeedId > 0) {
                await invoke('mark_all_read', { targetType: 'feed', id: this.selectedFeedId });
            } else if (this.selectedFolderId) {
                await invoke('mark_all_read', { targetType: 'folder', id: this.selectedFolderId });
            } else {
                return;
            }
            await this.refreshFolders();
            this.articles = this.articles.map((a) => (a.is_saved ? a : { ...a, is_read: true }));
        } catch (e) {
            console.error('Mark all read failed:', e);
        }
    }

    async addFeed(url: string, folderId: number | null = null) {
        this.isLoading = true;
        try {
            await invoke('add_feed', { url, folderId });
            await this.refreshFolders();
        } catch (e) {
            this.alert(`Error adding feed: ${e}`);
        } finally {
            this.isLoading = false;
        }
    }

    async createFolder(name: string) {
        try {
            await invoke('create_folder', { name });
            await this.refreshFolders();
        } catch (e) {
            console.error('Failed to create folder', e);
        }
    }

    async importOpml() {
        try {
            const selected = await open({
                multiple: false,
                filters: [{ name: 'OPML Files', extensions: ['opml', 'xml'] }],
            });
            if (selected && typeof selected === 'string') {
                this.isLoading = true;
                await invoke('import_opml', { path: selected });
                await this.refreshFolders();
            }
        } catch {
            this.alert('Failed to import OPML file.');
        } finally {
            this.isLoading = false;
        }
    }

    async exportOpml() {
        try {
            const opmlContent = await invoke<string>('export_opml');
            if (!opmlContent) return;
            const filePath = await save({
                filters: [{ name: 'OPML File', extensions: ['opml'] }],
                defaultPath: 'feeds.opml',
            });
            if (filePath) {
                await invoke('write_file', { path: filePath, content: opmlContent });
                this.alert('Export successful!');
            }
        } catch (e) {
            this.alert(`Failed to export OPML: ${e}`);
        }
    }

    async selectFolder(folderId: number) {
        if (this.selectedFolderId === folderId && !this.selectedFeedId) return;
        this.selectedFolderId = folderId;
        this.selectedFeedId = null;
        this.selectedArticle = null;
        this.isLoading = true;
        try {
            await this.reloadCurrentArticleList();
        } finally {
            this.isLoading = false;
        }
    }

    async selectFeed(feedId: number) {
        if (this.selectedFeedId === feedId) return;
        this.selectedFeedId = feedId;
        this.selectedFolderId = null;
        this.selectedArticle = null;
        this.isLoading = true;
        try {
            await this.reloadCurrentArticleList();
        } finally {
            this.isLoading = false;
        }
    }

    async loadMore() {
        if ((!this.selectedFeedId && !this.selectedFolderId) || !this.hasMore || this.isLoading)
            return;
        this.isLoading = true;
        const nextPage = this.page + 1;
        try {
            const result = await this.fetchPage(nextPage);
            if (result && result.length > 0) {
                this.articles = [...this.articles, ...result];
                this.page = nextPage;
                this.hasMore = result.length === this.pageSize;
            } else {
                this.hasMore = false;
            }
        } catch (e) {
            console.error('Failed to load more articles:', e);
        } finally {
            this.isLoading = false;
        }
    }

    private async fetchPage(page: number): Promise<Article[]> {
        const offset = page * this.pageSize;
        const sortDesc = this.sortOrder === 'desc';

        if (this.searchQuery.trim()) {
            return await invoke('search_articles', {
                query: this.searchQuery.trim(),
                limit: this.pageSize,
                offset,
                sortDesc,
            });
        }

        if (this.selectedFeedId === FEED_ID_LATEST) {
            const cutoff = Math.floor(Date.now() / 1000) - this.latestHours * 3600;
            return await invoke('get_latest_articles', {
                cutoffTimestamp: cutoff,
                limit: this.pageSize,
                offset,
                sortDesc,
            });
        } else if (this.selectedFeedId === FEED_ID_SAVED) {
            return await invoke('get_saved_articles', { limit: this.pageSize, offset, sortDesc });
        } else if (this.selectedFeedId) {
            return await invoke('get_articles_for_feed', {
                feedId: this.selectedFeedId,
                limit: this.pageSize,
                offset,
                sortDesc,
            });
        } else if (this.selectedFolderId) {
            return await invoke('get_articles_for_folder', {
                folderId: this.selectedFolderId,
                limit: this.pageSize,
                offset,
                sortDesc,
            });
        }
        return [];
    }

    selectArticle(article: Article) {
        this.selectedArticle = article;
        if (!article.is_read) {
            article.is_read = true;
            invoke('mark_article_read', { id: article.id, read: true }).catch((_e) => {
                article.is_read = false;
            });
            const feedId = article.feed_id;
            for (const folder of this.folders) {
                const feed = folder.feeds.find((f) => f.id === feedId);
                if (feed && feed.unread_count > 0) {
                    feed.unread_count--;
                    break;
                }
            }
        }
    }

    async toggleSaved(article: Article) {
        const newState = !article.is_saved;
        article.is_saved = newState;

        if (newState) {
            article.is_read = false;
            invoke('mark_article_read', { id: article.id, read: false }).catch(() => {});
        }

        try {
            await invoke('mark_article_saved', { id: article.id, isSaved: newState });
        } catch {
            article.is_saved = !newState;
        }
    }

    async renameFolder(id: number, newName: string) {
        try {
            await invoke('rename_folder', { id, newName });
            await this.refreshFolders();
        } catch (e) {
            console.error(e);
        }
    }

    async renameFeed(id: number, newName: string) {
        try {
            await invoke('rename_feed', { id, newName });
            await this.refreshFolders();
        } catch (e) {
            console.error(e);
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

    async deleteFeed(id: number) {
        this.confirm('Delete feed?', async () => {
            try {
                await invoke('delete_feed', { id });
                if (this.selectedFeedId === id) {
                    this.selectedFeedId = null;
                    this.articles = [];
                }
                await this.refreshFolders();
            } catch (e) {
                console.error(e);
            }
        });
    }

    async deleteFolder(id: number) {
        this.confirm('Delete folder and feeds?', async () => {
            try {
                await invoke('delete_folder', { id });
                await this.refreshFolders();
            } catch (e) {
                console.error(e);
            }
        });
    }

    async moveFeed(feedId: number, folderId: number) {
        try {
            await invoke('move_feed', { feedId, folderId });
            await this.refreshFolders();
        } catch (e) {
            console.error(e);
        }
    }

    setTheme(newTheme: Theme) {
        this.theme = newTheme;
    }

    async fetchFullContent(article: Article): Promise<string | null> {
        try {
            return await invoke<string>('get_article_content', { url: article.url });
        } catch {
            return null;
        }
    }

    openSettings() {
        this.showSettings = true;
    }

    closeSettings() {
        this.showSettings = false;
    }

    async saveSettings(newSettings: AppSettings) {
        try {
            await invoke('save_app_settings', { newSettings });
            this.settings = newSettings;
            this.closeSettings();
            // Optionally restart auto-update interval if changed
        } catch (e) {
            this.alert(`Failed to save settings: ${e}`);
        }
    }
}

export const appState = new AppState();
