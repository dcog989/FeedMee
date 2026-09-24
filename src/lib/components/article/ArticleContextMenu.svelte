<script lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { articleStore } from "$lib/store.svelte";
import type { Article } from "$lib/types";
import ContextMenu from "../ContextMenu.svelte";

let cmVisible = $state(false);
let cmX = $state(0);
let cmY = $state(0);
let cmArticle = $state<Article | null>(null);

export function show(event: MouseEvent, article: Article) {
  event.preventDefault();
  event.stopPropagation();
  cmArticle = article;
  cmX = event.clientX;
  cmY = event.clientY;
  cmVisible = true;
}

export function close() {
  cmVisible = false;
  cmArticle = null;
}

function openInBrowser() {
  if (cmArticle?.url) openUrl(cmArticle.url);
  close();
}

function toggleRead() {
  if (!cmArticle) return;
  const article = cmArticle;
  const newRead = !article.is_read;
  article.is_read = newRead;
  articleStore.adjustUnreadCount(article.feed_id, newRead ? -1 : 1);
  invoke("mark_article_read", { id: article.id, read: newRead }).catch((e) => {
    article.is_read = !newRead;
    articleStore.adjustUnreadCount(article.feed_id, newRead ? 1 : -1);
    console.error("mark_article_read failed:", e);
  });
  close();
}

function toggleSaved() {
  if (!cmArticle) return;
  articleStore.toggleSaved(cmArticle);
  close();
}
</script>

<ContextMenu x={cmX} y={cmY} visible={cmVisible} onClose={close}>
  <button type="button" onclick={openInBrowser}>Open in Browser</button>
  <button type="button" onclick={toggleRead}>
    {cmArticle?.is_read ? "Mark Unread" : "Mark Read"}
  </button>
  <button type="button" onclick={toggleSaved}>
    {cmArticle?.is_saved ? "Remove Bookmark" : "Bookmark"}
  </button>
</ContextMenu>
