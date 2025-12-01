import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import type { Article, Folder } from './types';

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

    async refreshAllFeeds() {
        this.isLoading = true;
        try {
            await invoke('refresh_all_feeds');
            await this.refreshFolders();
            // Reload current view
            if (this.selectedFeedId) {
                this.articles = [];
                this.page = 0;
                await this.selectFeed(this.selectedFeedId, true);
            } else if (this.selectedFolderId) {
                this.articles = [];
                this.page = 0;
                await this.selectFolder(this.selectedFolderId);
            }
        } catch (e) {
            console.error('Failed to refresh all feeds:', e);
            this.alert('Failed to refresh feeds.');
        } finally {
            this.isLoading = false;
        }
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
        this.articles = [];
        this.page = 0;
        this.hasMore = true;
        this.isLoading = true;

        try {
            const result = await this.fetchPage(0);
            this.articles = result || [];
            this.hasMore = (result?.length || 0) === this.pageSize;
        } catch (e) {
            console.error(`Failed to load articles for folder ${folderId}:`, e);
            this.articles = [];
        } finally {
            this.isLoading = false;
        }
    }

    async selectFeed(feedId: number, forceRefresh = false) {
        if (this.selectedFeedId === feedId && !forceRefresh) return;

        this.selectedFeedId = feedId;
        this.selectedFolderId = null;
        this.selectedArticle = null;
        this.articles = [];
        this.page = 0;
        this.hasMore = true;
        this.isLoading = true;

        try {
            // Trigger refresh in background if selecting a specific feed
            if (feedId > 0) {
                invoke('refresh_feed', { feedId }).then(() => {
                    this.refreshFolders(); // Update counts
                    // If we are still viewing this feed, prepend new articles
                    if (this.selectedFeedId === feedId) {
                        this.fetchPage(0).then(res => {
                            // Simple reload for now to ensure consistency
                            this.articles = res || [];
                        });
                    }
                });
            }

            const result = await this.fetchPage(0);

            if (this.selectedFeedId === feedId) {
                this.articles = result || [];
                this.hasMore = (result?.length || 0) === this.pageSize;
            }
        } catch (e) {
            console.error(`Failed to load articles for feed ${feedId}:`, e);
            if (this.selectedFeedId === feedId) this.articles = [];
        } finally {
            if (this.selectedFeedId === feedId) {
                this.isLoading = false;
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
        // Optimistic
        article.is_saved = newState;

        try {
            await invoke('mark_article_saved', { id: article.id, isSaved: newState });
        } catch (e) {
            console.error('Failed to toggle saved:', e);
            // Revert
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
