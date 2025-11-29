<script lang="ts">
    import { appState } from "$lib/store.svelte";

    let listContainer: HTMLElement;

    function onScroll() {
        if (!listContainer) return;

        const { scrollTop, scrollHeight, clientHeight } = listContainer;
        // Trigger load when within 100px of bottom
        if (scrollHeight - scrollTop <= clientHeight + 100) {
            appState.loadMore();
        }
    }
</script>

<section class="pane" bind:this={listContainer} onscroll={onScroll}>
    {#if appState.articles.length > 0}
        <ul class="article-list">
            {#each appState.articles as article (article.id)}
                <li>
                    <button class:selected={appState.selectedArticle?.id === article.id} onclick={() => appState.selectArticle(article)}>
                        <span class="title">{article.title}</span>
                        <div class="meta">
                            <span class="author">{article.author}</span>
                            <span class="date">{new Date(article.timestamp * 1000).toLocaleDateString()}</span>
                        </div>
                    </button>
                </li>
            {/each}
        </ul>
        {#if appState.isLoading}
            <div class="loading-more">Loading more...</div>
        {/if}
    {:else if appState.isLoading}
        <div class="loading">Loading articles...</div>
    {:else if appState.selectedFeedId}
        <div class="empty-state">
            <p>No articles in this feed.</p>
        </div>
    {:else}
        <div class="empty-state">
            <p>Select a feed to see articles.</p>
        </div>
    {/if}
</section>

<style>
    .pane {
        background-color: var(--bg-content);
        border-right: 1px solid var(--border-color);
        overflow-y: auto;
        height: 100%;
        box-sizing: border-box;
    }

    .article-list {
        list-style: none;
        margin: 0;
        padding: 0;
    }

    .loading,
    .empty-state,
    .loading-more {
        padding: 2rem 1rem;
        text-align: center;
        color: var(--text-secondary);
        font-size: 0.9rem;
    }

    button {
        display: block;
        width: 100%;
        padding: 1rem;
        text-align: left;
        border: none;
        border-bottom: 1px solid var(--border-color);
        background: transparent;
        cursor: pointer;
        color: var(--text-primary);
    }

    button:hover {
        background-color: var(--bg-hover);
    }

    button.selected {
        background-color: var(--bg-hover);
        border-left: 4px solid var(--bg-selected);
        padding-left: calc(1rem - 4px);
    }

    .title {
        display: block;
        font-weight: 600;
        margin-bottom: 0.4rem;
        font-size: 1rem;
        line-height: 1.3;
    }

    .meta {
        display: flex;
        justify-content: space-between;
        font-size: 0.8rem;
        color: var(--text-secondary);
    }
</style>
