<script lang="ts">
import { RefreshCw, X } from 'lucide-svelte';
import { tooltip } from '$lib/actions/tooltip.svelte';
import { FEED_FAILURE_LIMIT } from '$lib/feedRefresh.svelte';
import { navStore, refreshStore } from '$lib/store.svelte';
import type { Feed } from '$lib/types';
import { getFavicon, handleFaviconError } from '$lib/utils/favicon';

let { feed, isSelected = false }: { feed: Feed; isSelected?: boolean } = $props();
const disabled = $derived(feed.error_count >= FEED_FAILURE_LIMIT);
const disabledTooltip = $derived(`Feed disabled after ${feed.error_count} consecutive update failures. Click to retry.`);
</script>

<span class="feed-name-wrap">
    {#if feed.display_url}
        <img
            src={getFavicon(feed.display_url)}
            alt=""
            class="feed-favicon"
            loading="lazy"
            onerror={handleFaviconError}
        >
        <span class="feed-icon favicon-fallback-hidden" class:icon-selected={isSelected}>#</span>
    {:else}
        <span class="feed-icon" class:icon-selected={isSelected}>#</span>
    {/if}
    <span class="feed-name" class:feed-disabled={disabled} use:tooltip={disabled ? disabledTooltip : ''}>{feed.name}</span>
</span>

<button
    type="button"
    class="feed-action-area"
    onclick={(e) => {
        e.stopPropagation();
        refreshStore.requestRefreshFeed(feed.id);
    }}
    aria-label="Refresh feed"
>
    {#if refreshStore.isFeedUpdating(feed.id)}
        <div class="mini-spinner"></div>
    {:else if disabled || feed.has_error}
        <span class="error-badge" use:tooltip={disabled ? disabledTooltip : 'Feed update failed'}>
            <X size={10} color="white" />
        </span>
    {:else if feed.unread_count > 0}
        <span
            class="badge"
            class:badge-selected={isSelected}
            use:tooltip={refreshStore.isFeedFresh(feed.id)
                ? 'Already fresh!'
                : 'Click to refresh'}
            >{feed.unread_count}</span
        >
    {:else}
        <span
            class="refresh-icon"
            use:tooltip={refreshStore.isFeedFresh(feed.id)
                ? 'Already fresh!'
                : 'Refresh'}
        >
            <RefreshCw size={16} />
        </span>
    {/if}
</button>

<style>
.feed-favicon {
    width: 16px;
    height: 16px;
    border-radius: 2px;
}

.favicon-fallback-hidden {
    display: none;
}

.feed-action-area {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    min-width: 24px;
    height: 100%;
    cursor: pointer;
    padding-left: 8px;
    border: none;
    background: transparent;
    font: inherit;
    color: inherit;
}

.feed-name-wrap {
    display: flex;
    align-items: center;
    gap: 8px;
    white-space: nowrap;
    overflow: hidden;
}

.feed-name {
    overflow: hidden;
    text-overflow: ellipsis;
}

.feed-disabled {
    color: #d32f2f;
    text-decoration: line-through;
}

.feed-icon {
    color: var(--text-secondary);
    font-size: 0.8rem;
    opacity: 0.7;
    flex-shrink: 0;
}

.icon-selected {
    color: var(--bg-selected);
}

.badge-selected {
    background-color: var(--bg-selected);
    color: white;
}

.error-badge {
    width: 16px;
    height: 16px;
    background-color: #d32f2f;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
}

</style>
