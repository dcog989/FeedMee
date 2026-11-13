<script lang="ts">
import { createTagPopover } from "#lib/tagPopover.svelte.js";
import type { Article } from "#lib/types.js";
import ArticleContextMenu from "./article/ArticleContextMenu.svelte";
import ArticleList from "./article/ArticleList.svelte";
import ArticleSearch from "./article/ArticleSearch.svelte";
import ArticleToolbar from "./article/ArticleToolbar.svelte";
import TagPopover from "./TagPopover.svelte";

let ctxMenu: ArticleContextMenu;

const tags = createTagPopover();

function toggleTagManager(e: MouseEvent, article: Article) {
  e.stopPropagation();
  tags.toggle(e, article.id);
}

function onContextMenu(e: MouseEvent, article: Article) {
  ctxMenu.show(e, article);
}

function onListScroll() {
  ctxMenu.close();
  tags.close();
}
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") tags.close();
  }}
/>

<div class="pane-wrapper">
  <ArticleSearch />
  <ArticleToolbar />
  <ArticleList {onContextMenu} onTagToggle={toggleTagManager} tagArticleId={tags.articleId} onScroll={onListScroll} />
</div>

<ArticleContextMenu bind:this={ctxMenu} />

<TagPopover articleId={tags.articleId} x={tags.x} y={tags.y} onClose={tags.close} />
