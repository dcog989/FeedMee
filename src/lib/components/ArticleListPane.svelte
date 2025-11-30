<script lang="ts">
    import { appState } from "$lib/store.svelte";
    import type { Article } from "$lib/types";

    let listContainer: HTMLElement;

    function onScroll() {
        if (!listContainer) return;

        const { scrollTop, scrollHeight, clientHeight } = listContainer;
        if (scrollHeight - scrollTop <= clientHeight + 100) {
            appState.loadMore();
        }
    }

    function handleKeydown(e: KeyboardEvent, article: Article) {
        if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            appState.selectArticle(article);
        }
    }
</script>

<section class="pane" bind:this={listContainer} onscroll={onScroll}>
    {#if appState.articles.length > 0}
        <ul class="article-list">
            {#each appState.articles as article (article.id)}
                <li>
                    <div class="article-card" class:selected={appState.selectedArticle?.id === article.id} onclick={() => appState.selectArticle(article)} onkeydown={(e) => handleKeydown(e, article)} role="button" tabindex="0">
                        <span class="title" title={article.title}>{article.title}</span>

                        <div class="meta-line">
                            <span class="author">{article.author}</span>
                            <span class="date">{new Date(article.timestamp * 1000).toLocaleDateString()}</span>
                        </div>

                        <div class="actions-line">
                            <button
                                class="icon-btn"
                                class:active={article.is_saved}
                                onclick={(e) => {
                                    e.stopPropagation();
                                    appState.toggleSaved(article);
                                }}
                                title="Read Later"
                                aria-label="Read Later"
                            >
                                <svg width="14" height="14" viewBox="0 0 24 24" fill={article.is_saved ? "currentColor" : "none"} stroke="currentColor" stroke-width="2">
                                    <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"></path>
                                </svg>
                            </button>

                            <button class="icon-btn" onclick={(e) => e.stopPropagation()} title="Tag" aria-label="Tag">
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path>
                                    <line x1="7" y1="7" x2="7.01" y2="7"></line>
                                </svg>
                            </button>
                        </div>
                    </div>
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

    .article-card {
        display: block;
        width: 100%;
        padding: 0.8rem 1rem;
        text-align: left;
        border-bottom: 1px solid var(--border-color);
        background: transparent;
        cursor: pointer;
        color: var(--text-primary);
        overflow: hidden;
        outline: none;
        box-sizing: border-box;
    }

    .article-card:hover {
        background-color: var(--bg-hover);
    }

    .article-card:focus-visible {
        background-color: var(--bg-hover);
        box-shadow: inset 4px 0 0 var(--border-color);
    }

    .article-card.selected {
        background-color: var(--bg-hover);
        border-left: 4px solid var(--bg-selected);
        padding-left: calc(1rem - 4px);
    }

    .title {
        display: block;
        font-family: var(--font-serif);
        font-weight: 400;
        margin-bottom: 0.3rem;
        font-size: 0.95rem;
        line-height: 1.3;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .meta-line {
        display: flex;
        justify-content: space-between;
        font-size: 0.75rem;
        color: var(--text-secondary);
        margin-bottom: 0.4rem;
    }

    .author {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        max-width: 60%;
    }

    .actions-line {
        display: flex;
        gap: 0.8rem;
    }

    .icon-btn {
        display: flex;
        align-items: center;
        color: var(--text-secondary);
        opacity: 0.6;
        transition: all 0.2s;
        cursor: pointer;
        background: transparent;
        border: none;
        padding: 0;
    }

    .icon-btn:hover {
        opacity: 1;
        color: var(--text-primary);
    }

    .icon-btn.active {
        color: var(--bg-selected);
        opacity: 1;
    }
</style>
