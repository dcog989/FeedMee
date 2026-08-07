<script lang="ts">
import { ArrowUpDown, Bookmark, CalendarDays, CheckCheck, Clock } from 'lucide-svelte';
import { tooltip } from '$lib/actions/tooltip.svelte';
import { FEED_ID_LATEST, FEED_ID_SAVED, FEED_ID_TODAY, feedStore, settingsStore } from '$lib/store.svelte';
</script>

<div class="list-toolbar">
  <div class="toolbar-left">
    <button
      type="button"
      class="tool-btn"
      class:active={feedStore.selectedFeedId === FEED_ID_LATEST}
      onclick={() => feedStore.selectFeed(FEED_ID_LATEST)}
      use:tooltip={'Latest'}
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
      use:tooltip={'Read Later'}
      aria-label="Read Later"
    >
      <Bookmark size={18} />
    </button>
  </div>
  <div class="toolbar-right">
    <button
      type="button"
      class="tool-btn"
      onclick={() => settingsStore.setSortOrder(settingsStore.sortOrder === 'desc' ? 'asc' : 'desc')}
      use:tooltip={settingsStore.sortOrder === 'desc'
                ? 'Sort: Newest First'
                : 'Sort: Oldest First'}
      aria-label={settingsStore.sortOrder === 'desc' ? 'Sort Newest First' : 'Sort Oldest First'}
    >
      <ArrowUpDown size={20} />
    </button>

    <button
      type="button"
      class="tool-btn"
      onclick={() => feedStore.markAllRead()}
      use:tooltip={'Mark All Read'}
      aria-label="Mark All Read"
    >
      <CheckCheck size={20} />
    </button>
  </div>
</div>

<style>
.list-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-pane);
  flex-shrink: 0;
  height: 32px;
  box-sizing: border-box;
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.tool-btn {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border-radius: 4px;
  width: 32px;
  height: 32px;
}

.tool-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.tool-btn.active {
  color: var(--bg-selected);
}
</style>
