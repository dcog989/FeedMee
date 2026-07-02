import { invoke } from '@tauri-apps/api/core';
import type { AppState } from './storeTypes';

export function createNavigation(state: AppState) {
  async function markFeedReadOnExit(previousFeedId: number | null) {
    if (!state.settings.mark_feed_read_on_exit || !previousFeedId || previousFeedId <= 0) return;
    try {
      await invoke('mark_all_read', { targetType: 'feed', id: previousFeedId });
      const unreadCount = await invoke<number>('get_feed_unread_count', {
        feedId: previousFeedId,
      });
      for (const folder of state.folders) {
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

  function persistLastView(type: 'feed' | 'folder', id: number) {
    localStorage.setItem('lastViewType', type);
    localStorage.setItem('lastViewId', id.toString());
  }

  async function selectFolder(folderId: number) {
    if (state.selectedFolderId === folderId && !state.selectedFeedId) return;
    await markFeedReadOnExit(state.selectedFeedId);
    state.focusedPane = 'nav';
    state.searchQuery = '';
    state.selectedFolderId = folderId;
    state.selectedFeedId = null;
    state.selectedArticle = null;
    state.isLoadingArticles = true;
    persistLastView('folder', folderId);
    try {
      await state.reloadCurrentArticleList();
    } finally {
      state.isLoadingArticles = false;
    }
  }

  async function selectFeed(feedId: number) {
    if (state.selectedFeedId === feedId) return;
    await markFeedReadOnExit(state.selectedFeedId);
    state.focusedPane = 'nav';
    state.searchQuery = '';
    state.selectedFeedId = feedId;
    state.selectedFolderId = null;
    state.selectedArticle = null;
    state.isLoadingArticles = true;
    if (feedId > 0) persistLastView('feed', feedId);
    try {
      await state.reloadCurrentArticleList();
    } finally {
      state.isLoadingArticles = false;
    }
  }

  function getFlatNavItems(): { type: 'feed' | 'folder'; id: number }[] {
    const items: { type: 'feed' | 'folder'; id: number }[] = [];
    for (const folder of state.folders) {
      items.push({ type: 'folder', id: folder.id });
      if (state.expandedFolders.has(folder.id)) {
        for (const feed of folder.feeds) {
          items.push({ type: 'feed', id: feed.id });
        }
      }
    }
    return items;
  }

  function expandFolder(folderId: number) {
    const newSet = new Set(state.expandedFolders);
    if (state.settings.auto_collapse_folders) newSet.clear();
    newSet.add(folderId);
    state.expandedFolders = newSet;
  }

  function navUp() {
    const items = getFlatNavItems();
    if (items.length === 0) return;
    const currentIdx = items.findIndex(
      (i) =>
        (i.type === 'feed' && i.id === state.selectedFeedId) ||
        (i.type === 'folder' && i.id === state.selectedFolderId && !state.selectedFeedId),
    );
    const nextIdx = currentIdx <= 0 ? items.length - 1 : currentIdx - 1;
    const item = items[nextIdx];
    if (item.type === 'feed') selectFeed(item.id);
    else {
      expandFolder(item.id);
      selectFolder(item.id);
    }
  }

  function navDown() {
    const items = getFlatNavItems();
    if (items.length === 0) return;
    const currentIdx = items.findIndex(
      (i) =>
        (i.type === 'feed' && i.id === state.selectedFeedId) ||
        (i.type === 'folder' && i.id === state.selectedFolderId && !state.selectedFeedId),
    );
    const nextIdx = currentIdx < 0 || currentIdx >= items.length - 1 ? 0 : currentIdx + 1;
    const item = items[nextIdx];
    if (item.type === 'feed') selectFeed(item.id);
    else {
      expandFolder(item.id);
      selectFolder(item.id);
    }
  }

  function scrollSelectedIntoView(selector: string) {
    requestAnimationFrame(() => {
      document.querySelector<HTMLElement>(selector)?.scrollIntoView({ block: 'nearest' });
    });
  }

  function articleUp() {
    if (state.articles.length === 0) return;
    const idx = state.articles.findIndex((a) => a.id === state.selectedArticle?.id);
    const nextIdx = idx <= 0 ? 0 : idx - 1;
    state.selectArticle(state.articles[nextIdx]);
    scrollSelectedIntoView('.list-area .article-card.selected');
  }

  function articleDown() {
    if (state.articles.length === 0) return;
    const idx = state.articles.findIndex((a) => a.id === state.selectedArticle?.id);
    const nextIdx = idx < 0 ? 0 : Math.min(idx + 1, state.articles.length - 1);
    state.selectArticle(state.articles[nextIdx]);
    scrollSelectedIntoView('.list-area .article-card.selected');
  }

  return {
    selectFeed,
    selectFolder,
    navUp,
    navDown,
    articleUp,
    articleDown,
  };
}
