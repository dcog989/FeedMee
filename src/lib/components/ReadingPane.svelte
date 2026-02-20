<script lang="ts">
    import { tooltip } from '$lib/actions/tooltip.svelte';
    import { appState } from '$lib/store.svelte';
    import { openUrl } from '@tauri-apps/plugin-opener';
    import DOMPurify from 'dompurify';

    DOMPurify.addHook('afterSanitizeAttributes', (node: Element) => {
        if (node.tagName === 'A' && node.hasAttribute('href')) {
            const href = node.getAttribute('href') || '';
            node.setAttribute('title', href);
            node.setAttribute('target', '_blank');
            node.setAttribute('rel', 'noopener noreferrer');
        }
    });

    let fullContent = $state<string | null>(null);
    let isLoadingFull = $state(false);
    let loadError = $state(false);

    let displayHtml = $derived(
        fullContent
            ? DOMPurify.sanitize(fullContent)
            : appState.selectedArticle?.summary
              ? DOMPurify.sanitize(appState.selectedArticle.summary)
              : '',
    );
    let isSaved = $derived(appState.selectedArticle?.is_saved ?? false);

    $effect(() => {
        if (appState.selectedArticle) {
            fullContent = null;
            loadError = false;
        }
    });

    $effect(() => {
        if (appState.selectedArticle && appState.selectedArticle.is_saved) {
            const currentId = appState.selectedArticle.id;
            const timer = setTimeout(() => {
                if (
                    appState.selectedArticle?.id === currentId &&
                    appState.selectedArticle.is_saved
                ) {
                    appState.toggleSaved(appState.selectedArticle);
                }
            }, 5000);
            return () => clearTimeout(timer);
        }
    });

    async function loadFullContent() {
        if (!appState.selectedArticle) return;
        isLoadingFull = true;
        loadError = false;
        const content = await appState.fetchFullContent(appState.selectedArticle);
        if (content) {
            fullContent = content;
        } else {
            loadError = true;
        }
        isLoadingFull = false;
    }

    function formatDate(ts: number) {
        const d = new Date(ts * 1000);
        const datePart = d.toLocaleDateString('en-GB', {
            day: 'numeric',
            month: 'long',
            year: 'numeric',
        });
        const timePart = d.toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' });
        return `${datePart} / ${timePart}`;
    }

    // Intercept clicks on links
    async function handleContentClick(e: MouseEvent) {
        const target = e.target as HTMLElement;
        const anchor = target.closest('a');
        if (anchor && anchor.href) {
            e.preventDefault();
            await openUrl(anchor.href);
        }
    }
</script>

<main class="pane">
    {#if appState.selectedArticle}
        <article class="article-content">
            <header>
                <h1>
                    <a
                        href={appState.selectedArticle.url}
                        onclick={(e) => {
                            e.preventDefault();
                            openUrl(appState.selectedArticle!.url);
                        }}
                        rel="noopener noreferrer"
                        class="title-link">
                        {appState.selectedArticle.title}
                    </a>
                </h1>
                <div class="meta-row">
                    <div class="meta-left">
                        <span class="author">By {appState.selectedArticle.author}</span>
                        <span class="separator">•</span>
                        <span class="date">{formatDate(appState.selectedArticle.timestamp)}</span>
                    </div>

                    <div class="meta-actions">
                        <button
                            class="action-btn"
                            class:active={isSaved}
                            onclick={() =>
                                appState.selectedArticle &&
                                appState.toggleSaved(appState.selectedArticle)}
                            use:tooltip={'Read Later'}
                            aria-label="Read Later">
                            <svg
                                width="18"
                                height="18"
                                viewBox="0 0 24 24"
                                fill={isSaved ? 'currentColor' : 'none'}
                                stroke="currentColor"
                                stroke-width="2">
                                <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"></path>
                            </svg>
                        </button>

                        <button class="action-btn" use:tooltip={'Tag'} aria-label="Tag">
                            <svg
                                width="18"
                                height="18"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2">
                                <path
                                    d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"
                                ></path>
                                <line x1="7" y1="7" x2="7.01" y2="7"></line>
                            </svg>
                        </button>

                        <button
                            class="action-btn"
                            onclick={loadFullContent}
                            use:tooltip={'Load Full Content'}
                            disabled={isLoadingFull || !!fullContent}
                            aria-label="Load Full Content">
                            {#if isLoadingFull}
                                <span class="spinner"></span>
                            {:else}
                                <svg
                                    width="18"
                                    height="18"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2">
                                    <path
                                        d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
                                    ></path>
                                    <polyline points="14 2 14 8 20 8"></polyline>
                                    <line x1="16" y1="13" x2="8" y2="13"></line>
                                    <line x1="16" y1="17" x2="8" y2="17"></line>
                                    <polyline points="10 9 9 9 8 9"></polyline>
                                </svg>
                            {/if}
                        </button>
                    </div>
                </div>
            </header>

            {#if loadError}
                <div class="error-banner">
                    <svg
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        ><circle cx="12" cy="12" r="10"></circle><line
                            x1="12"
                            y1="8"
                            x2="12"
                            y2="12"></line
                        ><line x1="12" y1="16" x2="12.01" y2="16"></line></svg>
                    <span>Could not extract full content. Showing summary instead.</span>
                </div>
            {/if}

            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="summary" onclick={handleContentClick}>
                {@html displayHtml}
            </div>

            <footer class="article-footer">
                <a
                    href={appState.selectedArticle.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="original-link"
                    use:tooltip={appState.selectedArticle.url}
                    onclick={(e) => {
                        e.preventDefault();
                        openUrl(appState.selectedArticle!.url);
                    }}>
                    Read original article
                    <svg
                        width="12"
                        height="12"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2">
                        <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                        <polyline points="15 3 21 3 21 9"></polyline>
                        <line x1="10" y1="14" x2="21" y2="3"></line>
                    </svg>
                </a>
            </footer>
        </article>
    {:else}
        <div class="empty-state">
            <p class="empty-hint">Select an article to start reading</p>
            <img src="/feedmee.png" alt="" class="empty-logo" />
        </div>
    {/if}
</main>

<style>
    .pane {
        background-color: var(--bg-reading);
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
    }

    .title-link {
        color: var(--accent-muted);
        text-decoration: none;
    }

    .title-link:hover {
        text-decoration: underline;
        text-decoration-color: var(--accent-muted);
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
        padding: 6px;
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

    .action-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .error-banner {
        background-color: #ffeef0;
        color: #d32f2f;
        padding: 12px;
        border-radius: 6px;
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 1.5rem;
        font-size: 0.9rem;
    }

    :global([data-theme='dark']) .error-banner {
        background-color: #3e1b1b;
        color: #ff9999;
    }

    .summary {
        line-height: 1.8;
        font-size: 1.15rem;
        font-family: var(--font-body);
        color: var(--text-primary);
    }

    .summary :global(p) {
        margin-bottom: 1.5rem;
    }

    /* Force override for content that tries to set black text on dark bg */
    .summary :global(*) {
        color: inherit !important;
        background-color: transparent !important;
        max-width: 100% !important;
    }

    /* Restore link color */
    .summary :global(a) {
        color: #4899ec !important;
        text-decoration: none;
        cursor: pointer !important;
    }

    .summary :global(a:hover) {
        text-decoration: underline;
    }

    .summary :global(img) {
        max-width: 100%;
        height: auto;
        border-radius: 4px;
        /* Don't force transparent background on images, some might need white */
        background-color: initial !important;
    }

    .article-footer {
        margin-top: 3rem;
        padding-top: 1.5rem;
        border-top: 1px solid var(--border-color);
    }

    .original-link {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        color: var(--text-secondary);
        text-decoration: none;
        font-size: 0.9rem;
        padding: 8px 12px;
        border: 1px solid var(--border-color);
        border-radius: 4px;
        transition: background-color 0.2s;
    }

    .original-link:hover {
        background-color: var(--bg-hover);
        color: var(--text-primary);
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        height: 100%;
        gap: 2rem;
    }

    .empty-hint {
        color: var(--text-secondary);
        font-size: 1rem;
        margin: 0;
        opacity: 0.6;
    }

    .empty-logo {
        width: min(60%, 320px);
        opacity: 0.3;
        user-select: none;
        pointer-events: none;
    }

    .spinner {
        width: 14px;
        height: 14px;
        border: 2px solid var(--text-secondary);
        border-top-color: transparent;
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }
</style>
