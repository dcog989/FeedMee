<script lang="ts">
import { ArrowUpDown, Bookmark, CalendarDays, CheckCheck, Clock } from "@lucide/svelte";
import { tooltip } from "#lib/actions/tooltip.svelte.js";
import { FEED_ID_LATEST, FEED_ID_SAVED, FEED_ID_TODAY, feedStore, settingsStore } from "#lib/store.svelte.js";
import ArticleSearch from "./ArticleSearch.svelte";
</script>

<div class="list-toolbar app-toolbar">
  <div class="toolbar-left">
    <button
      type="button"
      class="tool-btn"
      class:active={feedStore.selectedFeedId === FEED_ID_LATEST}
      onclick={() => feedStore.selectFeed(FEED_ID_LATEST)}
      use:tooltip={"Latest"}
      aria-label="Latest"
    >
      <Clock size={18} />
    </button>
    <button
      type="button"
      class="tool-btn"
      class:active={feedStore.selectedFeedId === FEED_ID_TODAY}
      onclick={() => feedStore.selectFeed(FEED_ID_TODAY)}
      use:tooltip={"Today's articles"}
      aria-label="Today"
    >
      <CalendarDays size={18} />
    </button>
    <button
      type="button"
      class="tool-btn"
      class:active={feedStore.selectedFeedId === FEED_ID_SAVED}
      onclick={() => feedStore.selectFeed(FEED_ID_SAVED)}
      use:tooltip={"Read Later"}
      aria-label="Read Later"
    >
      <Bookmark size={18} />
    </button>
    <ArticleSearch />
  </div>
  <div class="toolbar-right">
    <button
      type="button"
      class="tool-btn"
      onclick={() => settingsStore.setSortOrder(settingsStore.sortOrder === "desc" ? "asc" : "desc")}
      use:tooltip={settingsStore.sortOrder === "desc" ? "Sort: Newest First" : "Sort: Oldest First"}
      aria-label={settingsStore.sortOrder === "desc" ? "Sort Newest First" : "Sort Oldest First"}
    >
      <ArrowUpDown size={18} />
    </button>

    {#if feedStore.selectedFeedId !== FEED_ID_SAVED}
      <button
        type="button"
        class="tool-btn"
        onclick={() => feedStore.markAllRead()}
        use:tooltip={"Mark All Read"}
        aria-label="Mark All Read"
      >
        <CheckCheck size={18} />
      </button>
    {/if}
  </div>
</div>

<style>
.list-toolbar {
  justify-content: space-between;
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.toolbar-left {
  flex: 1;
  min-width: 0;
}

.tool-btn.active {
  color: var(--bg-selected);
}
</style>
