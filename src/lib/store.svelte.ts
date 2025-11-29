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

    // Layout State (Default Widths)
    navWidth = $state(280);
    listWidth = $state(320);

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
        this.isLoading = true;

        try {
            // 1. Try to load existing articles
            let result = await invoke<Article[]>('get_articles_for_feed', { feedId });

            // 2. If empty, auto-refresh from network
            if (!result || result.length === 0) {
                await invoke('refresh_feed', { feedId });
                result = await invoke<Article[]>('get_articles_for_feed', { feedId });
            }

            if (this.selectedFeedId === feedId) {
                this.articles = result || [];
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

    selectArticle(article: Article) {
        this.selectedArticle = article;
    }

    setTheme(newTheme: Theme) {
        this.theme = newTheme;
    }
}

export const appState = new AppState();
