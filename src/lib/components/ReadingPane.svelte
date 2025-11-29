<script lang="ts">
    import { appState } from "$lib/store.svelte";
    import DOMPurify from "dompurify";

    // Sanitize content to prevent XSS from RSS feeds
    let sanitizedContent = $derived(appState.selectedArticle?.summary ? DOMPurify.sanitize(appState.selectedArticle.summary) : "");
</script>

<main class="pane">
    {#if appState.selectedArticle}
        <article class="article-content">
            <header>
                <h1>{appState.selectedArticle.title}</h1>
                <div class="meta">
                    <span class="author">By {appState.selectedArticle.author}</span>
                    <span class="date">{new Date(appState.selectedArticle.timestamp * 1000).toLocaleString()}</span>
                </div>
            </header>
            <!-- Safe to render sanitized HTML -->
            <div class="summary">
                {@html sanitizedContent}
            </div>
        </article>
    {:else}
        <div class="empty-state">
            <p>Select an article to start reading</p>
        </div>
    {/if}
</main>

<style>
    .pane {
        background-color: var(--bg-content);
        overflow-y: auto;
        height: 100%;
        padding: 2rem 3rem;
        box-sizing: border-box;
    }

    .article-content {
        max-width: 700px;
        margin: 0 auto;
    }

    h1 {
        font-size: 2.2rem;
        margin-bottom: 0.5rem;
        line-height: 1.2;
        color: var(--text-primary);
    }

    .meta {
        color: var(--text-secondary);
        margin-bottom: 2rem;
        font-size: 0.9rem;
        display: flex;
        gap: 1rem;
        border-bottom: 1px solid var(--border-color);
        padding-bottom: 1.5rem;
    }

    .summary {
        line-height: 1.8;
        font-size: 1.15rem;
        color: var(--text-primary);
    }

    /* Target content inside the HTML summary */
    .summary :global(p) {
        margin-bottom: 1.5rem;
    }

    .summary :global(a) {
        color: var(--bg-selected);
    }

    .summary :global(img) {
        max-width: 100%;
        height: auto;
        border-radius: 4px;
    }

    .empty-state {
        display: flex;
        justify-content: center;
        align-items: center;
        height: 100%;
        color: var(--text-secondary);
        font-size: 1.2rem;
    }
</style>
