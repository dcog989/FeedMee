<script lang="ts">
    import { appState, FEED_ID_LATEST, FEED_ID_SAVED } from '$lib/store.svelte';
    import { Clock, Bookmark } from 'lucide-svelte';
</script>

<div class="special-section">
    <div class="special-row">
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
            class="feed-item special"
            class:selected={appState.selectedFeedId === FEED_ID_LATEST}
            onclick={() => appState.selectFeed(FEED_ID_LATEST)}>
            <Clock size={16} class="feed-icon" />
            <span class="feed-name">Latest</span>
        </div>
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
            class="feed-item special"
            class:selected={appState.selectedFeedId === FEED_ID_SAVED}
            onclick={() => appState.selectFeed(FEED_ID_SAVED)}>
            <Bookmark size={16} class="feed-icon" />
            <span class="feed-name">Read Later</span>
        </div>
    </div>
</div>

<style>
    .special-section {
        padding: 4px 12px;
    }

    .special-row {
        display: flex;
        gap: 4px;
    }

    .feed-item {
        flex: 1;
        padding: 0.4rem 0.6rem;
        background: transparent;
        text-align: left;
        cursor: pointer;
        border-radius: 6px;
        font-size: 0.9rem;
        color: var(--text-primary);
        display: flex;
        align-items: center;
        justify-content: flex-start;
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

    .feed-name {
        overflow: hidden;
        text-overflow: ellipsis;
    }

    :global(.feed-icon) {
        color: var(--text-secondary);
    }
</style>
