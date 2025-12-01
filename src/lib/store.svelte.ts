import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import type { Article, Feed, Folder } from './types';

export type Theme = 'light' | 'dark' | 'sepia' | 'system';

// Special Feed IDs
export const FEED_ID_LATEST = -1;
export const FEED_ID_SAVED = -2;

class AppState {
    folders = $state<Folder[]>([]);
    articles = $state<Article[]>([]);
    selectedFeedId = $state<number | null>(null);
    selectedFolderId = $state<number | null>(null);
    selectedArticle = $state<Article | null>(null);
    isLoading = $state(false);
    theme = $state<Theme>('system');

    // Layout State
    navWidth = $state(280);
    listWidth = $state(320);

    page = 0;
    readonly pageSize = 50;
    hasMore = $state(true);

    // Settings
    latestHours = $state(24);

    // Refresh State
    lastRefreshed = new Map<number, number>(); // feedId -> timestamp
    lastRefreshAll = 0;
    updatingFeedIds = $state(new Set<number>());

    // Modal State (Confirm & Alert)
    modalState = $state<{
        isOpen: boolean;
        type: 'confirm' | 'alert';
        message: string;
        onConfirm: () => void;
    }>({
        isOpen: false,
        type: 'confirm',
        message: '',
        onConfirm: () => { }
    });

    constructor() {
        this.initStore();
    }

    private initStore() {
        const storedNav = localStorage.getItem('navWidth');
        const storedList = localStorage.getItem('listWidth');
        if (storedNav) this.navWidth = parseInt(storedNav);
        if (storedList) this.listWidth = parseInt(storedList);

        this.refreshFolders();

        $effect.root(() => {
            $effect(() => {
                localStorage.setItem('navWidth', this.navWidth.toString());
                localStorage.setItem('listWidth', this.listWidth.toString());
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

    async refreshAllFeeds() {
        // Debounce 2 minutes (120,000 ms)
        if (Date.now() - this.lastRefreshAll < 120000) {
            console.log('Refresh All debounced');
            return;
        }

        this.lastRefreshAll = Date.now();
        this.isLoading = true;

        const allFeeds: Feed[] = this.folders.flatMap(f => f.feeds);

        // We run updates in parallel (browser handles limits)
        // Each feed update will trigger a folder structure refresh on completion
        // to show progress in the UI (counts updating)
        const promises = allFeeds.map(feed => this.refreshSingleFeed(feed.id));

        try {
            await Promise.all(promises);
            // Final sync to ensure everything is correct
            await this.refreshFolders();

            // Reload current view if needed
            if (this.selectedFeedId) {
                await this.reloadCurrentArticleList();
            } else if (this.selectedFolderId) {
                await this.reloadCurrentArticleList();
            }
        } catch (e) {
            console.error('Failed to refresh all feeds:', e);
        } finally {
            this.isLoading = false;
        }
    }

    // Helper to refresh a single feed and track its loading state
    async refreshSingleFeed(feedId: number) {
        // Mark updating
        const newSet = new Set(this.updatingFeedIds);
        newSet.add(feedId);
        this.updatingFeedIds = newSet;

        try {
            await invoke('refresh_feed', { feedId });
            this.lastRefreshed.set(feedId, Date.now());
            // Update unread counts incrementally
            await this.refreshFolders();
        } catch (e) {
            console.error(`Failed to refresh feed ${feedId}:`, e);
        } finally {
            // Unmark updating
            const endSet = new Set(this.updatingFeedIds);
            endSet.delete(feedId);
            this.updatingFeedIds = endSet;
        }
    }

    async reloadCurrentArticleList() {
        this.articles = [];
        this.page = 0;
        const result = await this.fetchPage(0);
        this.articles = result || [];
        this.hasMore = (result?.length || 0) === this.pageSize;
    }

    async addFeed(url: string) {
        this.isLoading = true;
        try {
            await invoke('add_feed', { url, folderId: null });
            await this.refreshFolders();
        } catch (e) {
            console.error('Failed to add feed:', e);
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
                filters: [{ name: 'OPML Files', extensions: ['opml', 'xml'] }]
            });

            if (selected && typeof selected === 'string') {
                this.isLoading = true;
                await invoke('import_opml', { path: selected });
                await this.refreshFolders();
            }
        } catch (e) {
            console.error('OPML Import failed:', e);
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
                defaultPath: 'feeds.opml'
            });

            if (filePath) {
                await invoke('write_file', { path: filePath, content: opmlContent });
                this.alert('Export successful!');
            }
        } catch (e) {
            console.error('Export failed:', e);
            this.alert(`Failed to export OPML: ${e}`);
        }
    }

    async selectFolder(folderId: number) {
        if (this.selectedFolderId === folderId && !this.selectedFeedId) return;

        this.selectedFolderId = folderId;
        this.selectedFeedId = null;
        this.selectedArticle = null;

        await this.reloadCurrentArticleList();
    }

    async selectFeed(feedId: number, forceRefresh = false) {
        if (this.selectedFeedId === feedId && !forceRefresh) return;

        this.selectedFeedId = feedId;
        this.selectedFolderId = null;
        this.selectedArticle = null;

        await this.reloadCurrentArticleList();

        // Background Refresh with Debounce (5 minutes)
        if (feedId > 0 && !forceRefresh) {
            const last = this.lastRefreshed.get(feedId) || 0;
            if (Date.now() - last > 5 * 60 * 1000) {
                this.refreshSingleFeed(feedId).then(() => {
                    // If still selected, prepend/refresh list
                    if (this.selectedFeedId === feedId) {
                        this.reloadCurrentArticleList();
                    }
                });
            } else {
                console.log(`Feed ${feedId} refresh skipped (debounce)`);
            }
        }
    }

    async loadMore() {
        if ((!this.selectedFeedId && !this.selectedFolderId) || !this.hasMore || this.isLoading) return;

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

        if (this.selectedFeedId === FEED_ID_LATEST) {
            const cutoff = Math.floor(Date.now() / 1000) - (this.latestHours * 3600);
            return await invoke('get_latest_articles', { cutoffTimestamp: cutoff, limit: this.pageSize, offset });
        } else if (this.selectedFeedId === FEED_ID_SAVED) {
            return await invoke('get_saved_articles', { limit: this.pageSize, offset });
        } else if (this.selectedFeedId) {
            return await invoke('get_articles_for_feed', {
                feedId: this.selectedFeedId,
                limit: this.pageSize,
                offset
            });
        } else if (this.selectedFolderId) {
            return await invoke('get_articles_for_folder', {
                folderId: this.selectedFolderId,
                limit: this.pageSize,
                offset
            });
        }
        return [];
    }

    selectArticle(article: Article) {
        this.selectedArticle = article;
    }

    async toggleSaved(article: Article) {
        const newState = !article.is_saved;
        article.is_saved = newState;

        try {
            await invoke('mark_article_saved', { id: article.id, isSaved: newState });
        } catch (e) {
            console.error('Failed to toggle saved:', e);
            article.is_saved = !newState;
        }
    }

    async renameFolder(id: number, newName: string) {
        try {
            await invoke('rename_folder', { id, newName });
            await this.refreshFolders();
        } catch (e) {
            console.error('Rename failed:', e);
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
            }
        };
    }

    alert(message: string) {
        this.modalState = {
            isOpen: true,
            type: 'alert',
            message,
            onConfirm: () => {
                this.modalState.isOpen = false;
            }
        };
    }

    closeModal() {
        this.modalState.isOpen = false;
    }

    async deleteFeed(id: number) {
        this.confirm('Are you sure you want to delete this feed?', async () => {
            try {
                await invoke('delete_feed', { id });
                if (this.selectedFeedId === id) {
                    this.selectedFeedId = null;
                    this.articles = [];
                }
                await this.refreshFolders();
            } catch (e) {
                console.error('Delete feed failed:', e);
            }
        });
    }

    async deleteFolder(id: number) {
        this.confirm('Delete folder and all its feeds?', async () => {
            try {
                await invoke('delete_folder', { id });
                await this.refreshFolders();
            } catch (e) {
                console.error('Delete folder failed:', e);
            }
        });
    }

    async moveFeed(feedId: number, folderId: number) {
        try {
            await invoke('move_feed', { feedId, folderId });
            await this.refreshFolders();
        } catch (e) {
            console.error('Move feed failed:', e);
        }
    }

    setTheme(newTheme: Theme) {
        this.theme = newTheme;
    }

    async fetchFullContent(article: Article): Promise<string | null> {
        try {
            return await invoke<string>('get_article_content', { url: article.url });
        } catch (e) {
            console.error('Failed to load full content:', e);
            this.alert('Could not fetch full content for this article.');
            return null;
        }
    }
}

export const appState = new AppState();
