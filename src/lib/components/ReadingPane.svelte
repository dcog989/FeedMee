<script lang="ts">
    import { appState } from "$lib/store.svelte";
    import DOMPurify from "dompurify";

    DOMPurify.addHook("afterSanitizeAttributes", (node: Element) => {
        if (node.tagName === "A" && node.hasAttribute("href")) {
            const href = node.getAttribute("href") || "";
            node.setAttribute("title", href);
            node.setAttribute("target", "_blank");
            node.setAttribute("rel", "noopener noreferrer");
        }
    });

    let sanitizedContent = $derived(appState.selectedArticle?.summary ? DOMPurify.sanitize(appState.selectedArticle.summary) : "");
    let isSaved = $derived(appState.selectedArticle?.is_saved ?? false);
</script>

<main class="pane">
    {#if appState.selectedArticle}
        <article class="article-content">
            <header>
                <h1>{appState.selectedArticle.title}</h1>
                <div class="meta-row">
                    <div class="meta-left">
                        <span class="author">By {appState.selectedArticle.author}</span>
                        <span class="separator">•</span>
                        <span class="date">{new Date(appState.selectedArticle.timestamp * 1000).toLocaleString()}</span>
                    </div>

                    <div class="meta-actions">
                        <button class="action-btn" class:active={isSaved} onclick={() => appState.selectedArticle && appState.toggleSaved(appState.selectedArticle)} title="Read Later">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill={isSaved ? "currentColor" : "none"} stroke="currentColor" stroke-width="2">
                                <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"></path>
                            </svg>
                        </button>

                        <button class="action-btn" title="Tag this article (Coming Soon)">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path>
                                <line x1="7" y1="7" x2="7.01" y2="7"></line>
                            </svg>
                        </button>
                    </div>
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
        font-family: var(--font-serif);
        font-weight: 700;
        font-size: 2.2rem;
        margin-bottom: 0.8rem;
        line-height: 1.2;
        color: var(--text-primary);
    }

    .meta-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-bottom: 1px solid var(--border-color);
        padding-bottom: 1rem;
        margin-bottom: 2rem;
        color: var(--text-secondary);
        font-size: 0.9rem;
    }

    .meta-left {
        display: flex;
        gap: 0.5rem;
        align-items: center;
    }

    .separator {
        color: var(--border-color);
    }

    .meta-actions {
        display: flex;
        gap: 0.5rem;
    }

    .action-btn {
        background: transparent;
        border: none;
        color: var(--text-secondary);
        padding: 4px;
        border-radius: 4px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: all 0.2s;
    }

    .action-btn:hover {
        background-color: var(--bg-hover);
        color: var(--text-primary);
    }

    .action-btn.active {
        color: var(--bg-selected);
    }

    .summary {
        line-height: 1.8;
        font-size: 1.15rem;
        color: var(--text-primary);
    }

    .summary :global(p) {
        margin-bottom: 1.5rem;
    }

    .summary :global(a) {
        color: #4899ec;
        text-decoration: none;
    }

    .summary :global(a:hover) {
        text-decoration: underline;
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
