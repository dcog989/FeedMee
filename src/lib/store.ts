import { writable } from 'svelte/store';
import type { Article, Folder } from './types';

interface AppStore {
	folders: Folder[];
	articles: Article[];
	selectedFeedId: number | null;
	selectedArticle: Article | null;
}

export const store = writable<AppStore>({
	folders: [],
	articles: [],
	selectedFeedId: null,
	selectedArticle: null
});