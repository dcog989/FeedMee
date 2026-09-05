<script lang="ts">
import { tooltipState } from "$lib/actions/tooltip.svelte";
import type { Article } from "$lib/types";
import ArticleContextMenu from "./article/ArticleContextMenu.svelte";
import ArticleList from "./article/ArticleList.svelte";
import ArticleSearch from "./article/ArticleSearch.svelte";
import ArticleToolbar from "./article/ArticleToolbar.svelte";
import TagPopover from "./TagPopover.svelte";

let ctxMenu: ArticleContextMenu;

let tagArticleId = $state<number | null>(null);
let tagX = $state(0);
let tagY = $state(0);

function toggleTagManager(e: MouseEvent, article: Article) {
  e.stopPropagation();
  if (tagArticleId === article.id) {
    tagArticleId = null;
  } else {
    tooltipState.visible = false;
    tagArticleId = article.id;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    tagX = rect.left;
    tagY = rect.bottom + 4;
  }
}

function closeTagManager() {
  tagArticleId = null;
}

function onContextMenu(e: MouseEvent, article: Article) {
  ctxMenu.show(e, article);
}
</script>

<svelte:window onkeydown={(e) => { if (e.key === 'Escape') closeTagManager(); }} />

<div class="pane-wrapper">
  <ArticleSearch />
  <ArticleToolbar />
  <ArticleList {onContextMenu} onTagToggle={toggleTagManager} {tagArticleId} onScroll={() => ctxMenu.close()} />
</div>

<ArticleContextMenu bind:this={ctxMenu} />

<TagPopover articleId={tagArticleId} x={tagX} y={tagY} onClose={closeTagManager} />

<style>
.pane-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--bg-article, var(--bg-content));
  border-right: 1px solid var(--border-color);
}
</style>
