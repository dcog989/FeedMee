<script lang="ts">
    import { appState } from "$lib/store";
</script>

<section class="pane">
    {#if appState.isLoading}
        <div class="loading">Loading articles...</div>
    {:else if appState.articles.length > 0}
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
    .empty-state {
        padding: 3rem 1rem;
        text-align: center;
        color: var(--text-secondary);
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
        background-color: var(--bg-hover); /* List typically doesn't go blue, just darker grey */
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
