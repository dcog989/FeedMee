<script lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";
import { ExternalLink } from "lucide-svelte";
import { tooltip } from "$lib/actions/tooltip.svelte";
import { appState, articleStore } from "$lib/store.svelte";
import ArticleActions from "./article/ArticleActions.svelte";
import ArticleContent from "./article/ArticleContent.svelte";

let paneEl: HTMLElement | undefined = $state();
let fullContent = $state<string | null>(null);
let isLoadingFull = $state(false);
let loadError = $state(false);
let loadGen = $state(0);

let rawHtml = $derived(fullContent ?? articleStore.selectedArticle?.summary ?? "");
let heroImage = $derived(articleStore.selectedArticle?.image_url ?? "");
// Avoid duplicating an image the article body already embeds.
let showHero = $derived(Boolean(heroImage) && !rawHtml.toLowerCase().includes(heroImage.toLowerCase()));

$effect(() => {
  if (articleStore.selectedArticle) {
    fullContent = null;
    isLoadingFull = false;
    loadError = false;
    paneEl?.scrollTo(0, 0);
  }
});

async function loadFullContent() {
  if (!articleStore.selectedArticle) return;
  const gen = ++loadGen;
  isLoadingFull = true;
  loadError = false;
  const content = await articleStore.fetchFullContent(articleStore.selectedArticle);
  if (gen !== loadGen) return;
  if (content) {
    fullContent = stripDuplicateTitle(content, articleStore.selectedArticle.title);
  } else {
    loadError = true;
  }
  isLoadingFull = false;
}

function stripDuplicateTitle(html: string, articleTitle: string): string {
  const doc = new DOMParser().parseFromString(html, "text/html");
  const normalize = (s: string) => s.toLowerCase().replace(/\s+/g, " ").trim();
  const normalizedTitle = normalize(articleTitle);
  for (const el of doc.querySelectorAll("h1, h2")) {
    if (normalize(el.textContent ?? "").includes(normalizedTitle.slice(0, 30))) {
      el.remove();
      break;
    }
  }
  return doc.body.innerHTML;
}

function formatDate(ts: number) {
  const d = new Date(ts * 1000);
  const datePart = d.toLocaleDateString(undefined, {
    day: "numeric",
    month: "long",
    year: "numeric",
  });
  const timePart = d.toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  });
  return `${datePart}, ${timePart}`;
}

function getFeedDomain(feedId: number): string {
  for (const folder of appState.folders) {
    const feed = folder.feeds.find((f) => f.id === feedId);
    if (feed) {
      try {
        return new URL(feed.display_url).hostname.replace(/^www\./, "");
      } catch {
        return feed.name;
      }
    }
  }
  return "";
}
</script>

<main class="pane" bind:this={paneEl}>
  {#if articleStore.selectedArticle}
    <article class="article-content">
      <header>
        <h1>
          <a
            href={articleStore.selectedArticle.url}
            onclick={(e) => {
                            e.preventDefault();
                            if (articleStore.selectedArticle) openUrl(articleStore.selectedArticle.url);
                        }}
            rel="noopener noreferrer"
            class="title-link"
            use:tooltip={articleStore.selectedArticle.url}
          >
            {articleStore.selectedArticle.title}
          </a>
        </h1>
        <div class="meta-row">
          <div class="meta-left">
            <span class="author">{articleStore.selectedArticle.author}</span>
            <span class="feed-domain">{getFeedDomain(articleStore.selectedArticle.feed_id)}</span>
            <span class="date">{formatDate(articleStore.selectedArticle.timestamp)}</span>
          </div>

          <ArticleActions
            article={articleStore.selectedArticle}
            {isLoadingFull}
            hasFullContent={!!fullContent}
            onLoadFullContent={loadFullContent}
          />
        </div>
      </header>

      {#if showHero}
        <p>
          <img
            class="article-hero"
            src={heroImage}
            alt=""
            loading="lazy"
            onerror={(e) => {
              const img = e.currentTarget as HTMLImageElement;
              img.style.display = 'none';
            }}
          >
        </p>
      {/if}

      <ArticleContent {rawHtml} {loadError} articleUrl={articleStore.selectedArticle.url} />

      <footer class="article-footer">
        <a
          href={articleStore.selectedArticle.url}
          target="_blank"
          rel="noopener noreferrer"
          class="original-link"
          use:tooltip={articleStore.selectedArticle.url}
          onclick={(e) => {
                        e.preventDefault();
                        if (articleStore.selectedArticle) openUrl(articleStore.selectedArticle.url);
                    }}
        >
          Read original article
          <ExternalLink size={12} />
        </a>
      </footer>
    </article>
  {:else}
    <div class="empty-state">
      <p class="empty-hint">Select an article to start reading</p>
      <img src="/feedmee.png" alt="" class="empty-logo">
    </div>
  {/if}
</main>

<style>
.pane {
  background-color: var(--bg-article, var(--bg-reading));
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
  font-family: var(--font-title, var(--font-serif));
  font-weight: 700;
  font-size: 2.2rem;
  margin-bottom: 0.8rem;
  line-height: 1.2;
}

.title-link {
  color: var(--color-title, var(--accent-muted));
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

.article-hero {
  display: block;
  width: 100%;
  max-width: 100%;
  height: auto;
  border-radius: 6px;
  margin-bottom: 2rem;
}

.meta-left {
  display: flex;
  gap: 0.5rem;
  align-items: center;
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
</style>
