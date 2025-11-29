import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { Article, Folder } from './types';

export type Theme = 'light' | 'dark' | 'sepia' | 'system';

class AppState {
    folders = $state<Folder[]>([]);
    articles = $state<Article[]>([]);
    selectedFeedId = $state<number | null>(null);
    selectedArticle = $state<Article | null>(null);
    isLoading = $state(false);
    theme = $state<Theme>('system');

    // Layout State
    navWidth = $state(280);
    listWidth = $state(320);

    // Pagination State
    page = 0;
    readonly pageSize = 50;
    hasMore = $state(true);

    constructor() {
        this.refreshFolders();
    }

    async refreshFolders() {
        try {
            const result = await invoke<Folder[]>('get_folders_with_feeds');
            this.folders = result || [];
        } catch (e) {
            console.error('Failed to load folders:', e);
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
            alert('Failed to import OPML file.');
        } finally {
            this.isLoading = false;
        }
    }

    async selectFeed(feedId: number) {
        if (this.selectedFeedId === feedId) return;

        this.selectedFeedId = feedId;
        this.selectedArticle = null;
        this.articles = [];
        this.page = 0;
        this.hasMore = true;
        this.isLoading = true;

        try {
            // 1. Try to load existing articles (Page 0)
            let result = await this.fetchPage(feedId, 0);

            // 2. If empty, auto-refresh from network
            if (!result || result.length === 0) {
                await invoke('refresh_feed', { feedId });
                // Re-fetch page 0 after network refresh
                result = await this.fetchPage(feedId, 0);
            }

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
        if (!this.selectedFeedId || !this.hasMore || this.isLoading) return;

        this.isLoading = true;
        const nextPage = this.page + 1;

        try {
            const result = await this.fetchPage(this.selectedFeedId, nextPage);

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

    private async fetchPage(feedId: number, page: number): Promise<Article[]> {
        return await invoke<Article[]>('get_articles_for_feed', {
            feedId,
            limit: this.pageSize,
            offset: page * this.pageSize
        });
    }

    selectArticle(article: Article) {
        this.selectedArticle = article;
    }

    setTheme(newTheme: Theme) {
        this.theme = newTheme;
    }
}

export const appState = new AppState();
