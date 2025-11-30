<script lang="ts">
    import { appState, FEED_ID_LATEST, FEED_ID_SAVED } from "$lib/store.svelte";
</script>

<div class="special-section">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="feed-item special" class:selected={appState.selectedFeedId === FEED_ID_LATEST} onclick={() => appState.selectFeed(FEED_ID_LATEST)}>
        <span class="feed-name-wrap">
            <svg class="feed-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>
            <span class="feed-name">Latest</span>
        </span>
    </div>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="feed-item special" class:selected={appState.selectedFeedId === FEED_ID_SAVED} onclick={() => appState.selectFeed(FEED_ID_SAVED)}>
        <span class="feed-name-wrap">
            <svg class="feed-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"></path>
            </svg>
            <span class="feed-name">Read Later</span>
        </span>
    </div>
</div>

<style>
    .special-section {
        padding: 4px 12px;
    }

    .feed-item {
        width: 100%;
        padding: 0.4rem 0.6rem;
        background: transparent;
        text-align: left;
        cursor: pointer;
        border-radius: 6px;
        font-size: 0.9rem;
        color: var(--text-primary);
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 8px;
        border-left: 3px solid transparent;
        box-sizing: border-box;
    }

    .feed-item.special {
        font-weight: 500;
        margin-bottom: 2px;
    }

    .feed-item:hover {
        background-color: var(--bg-hover);
    }

    .feed-item.selected {
        background-color: var(--bg-selected-muted);
        color: var(--text-primary);
        border-left-color: var(--bg-selected);
        font-weight: 500;
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
</style>
