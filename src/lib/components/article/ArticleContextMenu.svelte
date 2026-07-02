<script lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { articleStore } from '$lib/store.svelte';
import type { Article } from '$lib/types';
import ContextMenu from '../ContextMenu.svelte';

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
    const newRead = !cmArticle.is_read;
    cmArticle.is_read = newRead;
    invoke('mark_article_read', { id: cmArticle.id, read: newRead }).catch(() => {
        if (cmArticle) cmArticle.is_read = !newRead;
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
        {cmArticle?.is_read ? 'Mark Unread' : 'Mark Read'}
    </button>
    <button type="button" onclick={toggleSaved}>
        {cmArticle?.is_saved ? 'Remove Bookmark' : 'Bookmark'}
    </button>
</ContextMenu>
