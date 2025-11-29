import { invoke } from '@tauri-apps/api/core';
import type { Article, Folder } from './types';

export type Theme = 'light' | 'dark' | 'sepia' | 'system';

class AppState {
	folders = $state<Folder[]>([]);
	articles = $state<Article[]>([]);
	selectedFeedId = $state<number | null>(null);
	selectedArticle = $state<Article | null>(null);
	isLoading = $state(false);
	theme = $state<Theme>('system');

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

	async selectFeed(feedId: number) {
		if (this.selectedFeedId === feedId) return;

		this.selectedFeedId = feedId;
		this.selectedArticle = null;
		this.articles = [];
		this.isLoading = true;

		try {
			const result = await invoke<Article[]>('get_articles_for_feed', { feedId });

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
