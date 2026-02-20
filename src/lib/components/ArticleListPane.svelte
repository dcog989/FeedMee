<script lang="ts">
    import { tooltip } from '$lib/actions/tooltip.svelte';
    import { appState } from '$lib/store.svelte';
    import type { Article } from '$lib/types';
    import { ArrowUpDown, Bookmark, CheckCheck } from 'lucide-svelte';

    let listContainer: HTMLElement;

    function onScroll() {
        if (!listContainer) return;
        const { scrollTop, scrollHeight, clientHeight } = listContainer;
        if (scrollHeight - scrollTop <= clientHeight + 100) {
            appState.loadMore();
        }
    }

    function handleKeydown(e: KeyboardEvent, article: Article) {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            appState.selectArticle(article);
        }
    }
</script>

<div class="pane-wrapper">
    <div class="list-toolbar">
        <button
            class="tool-btn"
            onclick={() => appState.setSortOrder(appState.sortOrder === 'desc' ? 'asc' : 'desc')}
            use:tooltip={appState.sortOrder === 'desc'
                ? 'Sort: Newest First'
                : 'Sort: Oldest First'}
            aria-label={appState.sortOrder === 'desc' ? 'Sort Newest First' : 'Sort Oldest First'}>
            <ArrowUpDown size={20} />
        </button>

        <button
            class="tool-btn"
            onclick={() => appState.markAllRead()}
            use:tooltip={'Mark All Read'}
            aria-label="Mark All Read">
            <CheckCheck size={20} />
        </button>
    </div>

    <section class="pane" bind:this={listContainer} onscroll={onScroll}>
        {#if appState.articles.length > 0}
            <ul class="article-list">
                {#each appState.articles as article (article.id)}
                    <li>
                        <div
                            class="article-card"
                            class:selected={appState.selectedArticle?.id === article.id}
                            class:unread={!article.is_read}
                            onclick={() => appState.selectArticle(article)}
                            onkeydown={(e) => handleKeydown(e, article)}
                            role="button"
                            tabindex="0">
                            <span class="title" title={article.title}>{article.title}</span>

                            <div class="meta-line">
                                <div class="meta-left">
                                    <span class="date"
                                        >{new Date(
                                            article.timestamp * 1000,
                                        ).toLocaleDateString()}</span>
                                    <span class="separator">•</span>
                                    <span class="author">{article.author}</span>
                                </div>

                                <div class="actions">
                                    <button
                                        class="icon-btn"
                                        class:active={article.is_saved}
                                        onclick={(e) => {
                                            e.stopPropagation();
                                            appState.toggleSaved(article);
                                        }}
                                        use:tooltip={'Read Later'}
                                        aria-label="Read Later">
                                        <Bookmark
                                            size={14}
                                            fill={article.is_saved ? 'currentColor' : 'none'} />
                                    </button>
                                </div>
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
        {:else if appState.selectedFolderId}
            <div class="empty-state">
                <p>No unread articles in this folder.</p>
            </div>
        {:else}
            <div class="empty-state">
                <p>Select a feed to see articles.</p>
            </div>
        {/if}
    </section>
</div>

<style>
    .pane-wrapper {
        display: flex;
        flex-direction: column;
        height: 100%;
        background-color: var(--bg-content);
        border-right: 1px solid var(--border-color);
    }

    .list-toolbar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 4px 8px; /* Reduced */
        border-bottom: 1px solid var(--border-color);
        background: var(--bg-pane);
        flex-shrink: 0;
        height: 32px;
        box-sizing: border-box;
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

    .pane {
        flex: 1;
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
        color: var(--text-secondary);
        overflow: hidden;
        outline: none;
        box-sizing: border-box;
    }

    .article-card.unread {
        color: var(--text-primary);
        font-weight: 400;
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
        color: var(--text-primary);
    }

    .title {
        display: block;
        font-family: var(--font-serif);
        margin-bottom: 0.3rem;
        font-size: 0.95rem;
        font-weight: 300;
        line-height: 1.3;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        /* Strikethrough for read articles (which are NOT .unread) */
        text-decoration: line-through;
        opacity: 0.7;
    }

    .unread .title {
        text-decoration: none; /* Reset for unread */
        opacity: 1;
    }

    .meta-line {
        display: flex;
        align-items: center;
        justify-content: space-between;
        font-size: 0.75rem;
        color: var(--text-secondary);
        margin-top: 0.4rem;
        font-weight: 400;
    }

    .meta-left {
        display: flex;
        gap: 6px;
        align-items: center;
        overflow: hidden;
    }

    .separator {
        opacity: 0.5;
    }

    .author {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .actions {
        display: flex;
        gap: 8px;
    }

    .icon-btn {
        display: flex;
        align-items: center;
        color: var(--text-secondary);
        opacity: 0.4;
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
