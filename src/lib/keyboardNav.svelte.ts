import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { AppState } from './storeTypes';
import { shortcutManager } from './utils/shortcuts';

export function registerShortcuts(state: AppState) {
    shortcutManager.register({
        id: 'settings',
        command: 'settings',
        defaultKey: ',',
        description: 'Open settings',
        category: 'General',
        handler: () => state.openSettings(),
    });

    shortcutManager.register({
        id: 'add-feed',
        command: 'add-feed',
        defaultKey: 'n',
        description: 'Add new feed',
        category: 'General',
        handler: () => {
            state.showAddDialog = true;
        },
    });

    shortcutManager.register({
        id: 'refresh-all',
        command: 'refresh-all',
        defaultKey: 'r',
        description: 'Refresh all feeds',
        category: 'Feeds',
        handler: () => state.refreshAllFeeds(),
    });

    shortcutManager.register({
        id: 'focus-search',
        command: 'focus-search',
        defaultKey: '/',
        description: 'Focus search',
        category: 'General',
        handler: () => {
            const searchInput = document.querySelector('.search-wrapper input') as HTMLInputElement;
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
            if (state.selectedArticle) state.toggleSaved(state.selectedArticle);
        },
    });

    shortcutManager.register({
        id: 'mark-read',
        command: 'mark-read',
        defaultKey: 'm',
        description: 'Mark as read/unread',
        category: 'Articles',
        handler: async () => {
            if (state.selectedArticle) {
                const article = state.selectedArticle;
                const newReadState = !article.is_read;
                article.is_read = newReadState;
                await invoke('mark_article_read', { id: article.id, read: newReadState });
            }
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
            state.folders.forEach((f) => newSet.add(f.id));
            state.expandedFolders = newSet;
        },
    });

    shortcutManager.register({
        id: 'collapse-all',
        command: 'collapse-all',
        defaultKey: 'c',
        description: 'Collapse all folders',
        category: 'Feeds',
        handler: () => {
            state.expandedFolders = new Set<number>();
        },
    });

    shortcutManager.register({
        id: 'open-article',
        command: 'open-article',
        defaultKey: 'enter',
        description: 'Open article in browser',
        category: 'Articles',
        handler: () => {
            if (state.selectedArticle) openUrl(state.selectedArticle.url);
        },
    });
}

export function setupKeyHandler(state: AppState) {
    window.addEventListener('keydown', (e) => {
        if (state.showSettings) return;

        const tag = (e.target as HTMLElement)?.tagName?.toLowerCase();
        const isInput =
            tag === 'input' || tag === 'textarea' || (e.target as HTMLElement)?.isContentEditable;
        if (isInput) return;

        switch (e.key) {
            case 'ArrowLeft':
                e.preventDefault();
                if (state.focusedPane === 'reading') state.focusedPane = 'list';
                else if (state.focusedPane === 'list') state.focusedPane = 'nav';
                return;
            case 'ArrowRight':
                e.preventDefault();
                if (state.focusedPane === 'nav') state.focusedPane = 'list';
                else if (state.focusedPane === 'list' && state.selectedArticle)
                    state.focusedPane = 'reading';
                return;
            case 'ArrowUp':
                e.preventDefault();
                if (state.focusedPane === 'nav') state.navUp();
                else if (state.focusedPane === 'list') state.articleUp();
                else if (state.focusedPane === 'reading') {
                    document
                        .querySelector<HTMLElement>('.reading-area .pane')
                        ?.scrollBy({ top: -80, behavior: 'smooth' });
                }
                return;
            case 'ArrowDown':
                e.preventDefault();
                if (state.focusedPane === 'nav') state.navDown();
                else if (state.focusedPane === 'list') state.articleDown();
                else if (state.focusedPane === 'reading') {
                    document
                        .querySelector<HTMLElement>('.reading-area .pane')
                        ?.scrollBy({ top: 80, behavior: 'smooth' });
                }
                return;
        }

        shortcutManager.handleKeyEvent(e);
    });
}
