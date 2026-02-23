import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import type { AppState } from './storeTypes';

export function createFeedActions(state: AppState) {
    async function markAllRead() {
        try {
            if (state.selectedFeedId === -1) {
                await invoke('mark_all_read', { targetType: 'global', id: 0 });
            } else if (state.selectedFeedId && state.selectedFeedId > 0) {
                await invoke('mark_all_read', { targetType: 'feed', id: state.selectedFeedId });
            } else if (state.selectedFolderId) {
                await invoke('mark_all_read', { targetType: 'folder', id: state.selectedFolderId });
            } else {
                return;
            }
            await state.refreshFolders();
            state.articles = state.articles.map((a) => (a.is_saved ? a : { ...a, is_read: true }));
        } catch (e) {
            console.error('Mark all read failed:', e);
        }
    }

    async function addFeed(url: string, folderId: number | null = null) {
        state.isLoadingArticles = true;
        try {
            await invoke('add_feed', { url, folderId });
            await state.refreshFolders();
        } catch (e) {
            state.alert(`Error adding feed: ${e}`);
        } finally {
            state.isLoadingArticles = false;
        }
    }

    async function createFolder(name: string) {
        try {
            await invoke('create_folder', { name });
            await state.refreshFolders();
        } catch (e) {
            console.error('Failed to create folder', e);
        }
    }

    async function importOpml() {
        try {
            const selected = await open({
                multiple: false,
                filters: [{ name: 'OPML Files', extensions: ['opml', 'xml'] }],
            });
            if (selected && typeof selected === 'string') {
                state.isLoadingArticles = true;
                await invoke('import_opml', { path: selected });
                await state.refreshFolders();
            }
        } catch {
            state.alert('Failed to import OPML file.');
        } finally {
            state.isLoadingArticles = false;
        }
    }

    async function exportOpml() {
        try {
            const opmlContent = await invoke<string>('export_opml');
            if (!opmlContent) return;
            const filePath = await save({
                filters: [{ name: 'OPML File', extensions: ['opml'] }],
                defaultPath: 'feeds.opml',
            });
            if (filePath) {
                await invoke('write_file', { path: filePath, content: opmlContent });
                state.alert('Export successful!');
            }
        } catch (e) {
            state.alert(`Failed to export OPML: ${e}`);
        }
    }

    async function renameFolder(id: number, newName: string) {
        try {
            await invoke('rename_folder', { id, newName });
            await state.refreshFolders();
        } catch (e) {
            console.error(e);
        }
    }

    async function renameFeed(id: number, newName: string) {
        try {
            await invoke('rename_feed', { id, newName });
            await state.refreshFolders();
        } catch (e) {
            console.error(e);
        }
    }

    async function deleteFeed(id: number) {
        state.confirm('Delete feed?', async () => {
            try {
                await invoke('delete_feed', { id });
                if (state.selectedFeedId === id) {
                    state.selectedFeedId = null;
                    state.articles = [];
                }
                await state.refreshFolders();
            } catch (e) {
                console.error(e);
            }
        });
    }

    async function deleteFolder(id: number) {
        state.confirm('Delete folder and feeds?', async () => {
            try {
                await invoke('delete_folder', { id });
                await state.refreshFolders();
            } catch (e) {
                console.error(e);
            }
        });
    }

    async function moveFeed(feedId: number, folderId: number) {
        try {
            await invoke('move_feed', { feedId, folderId });
            await state.refreshFolders();
        } catch (e) {
            console.error(e);
        }
    }

    return {
        markAllRead,
        addFeed,
        createFolder,
        importOpml,
        exportOpml,
        renameFolder,
        renameFeed,
        deleteFeed,
        deleteFolder,
        moveFeed,
    };
}
