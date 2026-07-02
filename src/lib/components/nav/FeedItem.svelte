<script lang="ts">
import { RefreshCw, X } from 'lucide-svelte';
import { tooltip } from '$lib/actions/tooltip.svelte';
import { appState } from '$lib/store.svelte';
import type { Feed } from '$lib/types';
import { getFavicon, handleFaviconError } from '$lib/utils/favicon';

let { feed, isSelected = false }: { feed: Feed; isSelected?: boolean } = $props();
</script>

<span class="feed-name-wrap">
    {#if feed.url}
        <img
            src={getFavicon(feed.url)}
            alt=""
            class="feed-favicon"
            loading="lazy"
            onerror={handleFaviconError}
        >
        <span class="feed-icon favicon-fallback-hidden" class:icon-selected={isSelected}>#</span>
    {:else}
        <span class="feed-icon" class:icon-selected={isSelected}>#</span>
    {/if}
    <span class="feed-name">{feed.name}</span>
</span>

<button
    type="button"
    class="feed-action-area"
    onclick={(e) => {
        e.stopPropagation();
        appState.requestRefreshFeed(feed.id);
    }}
    aria-label="Refresh feed"
>
    {#if appState.isFeedUpdating(feed.id)}
        <div class="mini-spinner"></div>
    {:else if feed.has_error}
        <span class="error-badge" use:tooltip={'Feed update failed'}>
            <X size={10} color="white" />
        </span>
    {:else if feed.unread_count > 0}
        <span
            class="badge"
            class:badge-selected={isSelected}
            use:tooltip={appState.isFeedFresh(feed.id)
                ? 'Already fresh!'
                : 'Click to refresh'}
            >{feed.unread_count}</span
        >
    {:else}
        <span
            class="refresh-icon"
            use:tooltip={appState.isFeedFresh(feed.id)
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

.feed-icon {
    color: var(--text-secondary);
    font-size: 0.8rem;
    opacity: 0.7;
    flex-shrink: 0;
}

.icon-selected {
    color: var(--bg-selected);
}

.badge {
    background-color: var(--text-secondary);
    color: var(--bg-pane);
    font-size: 0.75rem;
    padding: 1px 6px;
    border-radius: 10px;
    font-weight: 600;
    min-width: 16px;
    text-align: center;
    flex-shrink: 0;
}

.badge:hover {
    background-color: var(--bg-selected);
    color: white;
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

.refresh-icon {
    color: var(--text-secondary);
    opacity: 0.5;
    transition: opacity 0.2s;
    display: flex;
    align-items: center;
}

.refresh-icon:hover {
    opacity: 1;
    color: var(--text-primary);
}

.mini-spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--text-secondary);
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    flex-shrink: 0;
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}
</style>
