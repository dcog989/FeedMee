import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { FEED_ID_LATEST, FEED_ID_TODAY } from "./store.svelte";
import type { FeedStore } from "./storeTypes";

export function createFeedActions(state: FeedStore) {
  async function markAllRead() {
    try {
      if (state.selectedFeedId === FEED_ID_LATEST || state.selectedFeedId === FEED_ID_TODAY) {
        await invoke("mark_all_read", { targetType: "global", id: 0 });
      } else if (state.selectedFeedId && state.selectedFeedId > 0) {
        await invoke("mark_all_read", {
          targetType: "feed",
          id: state.selectedFeedId,
        });
      } else if (state.selectedFolderId) {
        await invoke("mark_all_read", {
          targetType: "folder",
          id: state.selectedFolderId,
        });
      } else {
        return;
      }
      await state.refreshFolders();
      state.articles = state.articles.map((a) => (a.is_saved ? a : { ...a, is_read: true }));
    } catch (e) {
      console.error("Mark all read failed:", e);
    }
  }

  async function addFeed(url: string, folderId: number | null = null) {
    state.isLoadingArticles = true;
    try {
      await invoke("add_feed", { url, folderId });
      await state.refreshFolders();
    } finally {
      state.isLoadingArticles = false;
    }
  }

  async function createFolder(name: string) {
    if (state.folders.some((f) => f.id !== 0 && f.name.toLowerCase() === name.toLowerCase())) {
      state.alert("A folder with that name already exists.");
      return;
    }
    try {
      await invoke("create_folder", { name });
      await state.refreshFolders();
    } catch (e) {
      console.error("Failed to create folder", e);
    }
  }

  async function importOpml() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "OPML Files", extensions: ["opml", "xml"] }],
    }).catch(() => null);

    if (!selected || typeof selected !== "string") return;

    state.isLoadingArticles = true;
    try {
      await invoke("import_opml", { path: selected });
      await state.refreshFolders();
      await state.refreshAllFeeds();
      await state.reloadCurrentArticleList();
    } catch {
      state.alert("Failed to import OPML file.");
    } finally {
      state.isLoadingArticles = false;
    }
  }

  async function exportOpml() {
    try {
      const filePath = await save({
        filters: [{ name: "OPML File", extensions: ["opml"] }],
        defaultPath: "feeds.opml",
      });
      if (!filePath) return;
      await invoke("export_opml", { path: filePath });
      state.alert("Export successful!");
    } catch (e) {
      state.alert(`Failed to export OPML: ${e}`);
    }
  }

  async function renameFolder(id: number, newName: string) {
    try {
      await invoke("rename_folder", { id, newName });
      await state.refreshFolders();
    } catch (e) {
      console.error(e);
    }
  }

  async function renameFeed(id: number, newName: string, newUrl: string) {
    try {
      await invoke("rename_feed", { id, newName, newUrl });
      await state.refreshFolders();
    } catch (e) {
      console.error(e);
    }
  }

  function removeStaleRefreshEntries(feedId: number) {
    state.lastRefreshed.delete(feedId);
    state.persistLastRefreshed();
  }

  async function deleteFeed(id: number) {
    state.confirm("Delete feed?", async () => {
      try {
        await invoke("delete_feed", { id });
        removeStaleRefreshEntries(id);
        const wasSelectedFeed = state.selectedFeedId === id;
        if (wasSelectedFeed) {
          state.selectedFeedId = null;
          state.articles = [];
        }
        await state.refreshFolders();
        if (wasSelectedFeed) {
          await state.selectFeed(FEED_ID_LATEST);
        } else if (state.selectedFeedId !== null || state.selectedFolderId !== null) {
          // A folder view may have included this feed's articles; drop them.
          await state.reloadCurrentArticleList();
        }
      } catch (e) {
        console.error(e);
      }
    });
  }

  async function deleteFolder(id: number) {
    state.confirm("Delete folder and feeds?", async () => {
      try {
        await invoke("delete_folder", { id });
        const folder = state.folders.find((f) => f.id === id);
        const removedFeedIds = new Set(folder?.feeds.map((f) => f.id));
        const selectionGone =
          state.selectedFolderId === id || (state.selectedFeedId !== null && removedFeedIds.has(state.selectedFeedId));
        if (folder) {
          for (const feed of folder.feeds) {
            removeStaleRefreshEntries(feed.id);
          }
        }
        await state.refreshFolders();
        if (selectionGone) await state.selectFeed(FEED_ID_LATEST);
      } catch (e) {
        console.error(e);
      }
    });
  }

  async function moveFeed(feedId: number, folderId: number | null) {
    try {
      await invoke("move_feed", { feedId, folderId });
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
