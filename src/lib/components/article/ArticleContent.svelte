<script module lang="ts">
import DOMPurify from 'dompurify';

DOMPurify.addHook('afterSanitizeAttributes', (node: Element) => {
  if (node.tagName === 'A' && node.hasAttribute('href')) {
    const href = node.getAttribute('href') || '';
    node.setAttribute('title', href);
    node.setAttribute('target', '_blank');
    node.setAttribute('rel', 'noopener noreferrer');
  }
  if (node.tagName === 'IMG') {
    const dataOriginal = node.getAttribute('data-original');
    const dataSrc = node.getAttribute('data-src');
    if (dataOriginal) {
      node.setAttribute('src', dataOriginal);
      node.removeAttribute('data-original');
    } else if (dataSrc) {
      node.setAttribute('src', dataSrc);
      node.removeAttribute('data-src');
    }
    const dataSrcset = node.getAttribute('data-srcset');
    if (dataSrcset) {
      node.setAttribute('srcset', dataSrcset);
      node.removeAttribute('data-srcset');
    }
    node.setAttribute('loading', 'lazy');
  }
});
</script>

<script lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener';
import { CircleAlert } from 'lucide-svelte';

let {
  rawHtml = '',
  loadError = false,
  articleUrl = '',
}: {
  rawHtml?: string;
  loadError?: boolean;
  articleUrl?: string;
} = $props();

let displayHtml = $derived(rawHtml ? DOMPurify.sanitize(rawHtml) : '');

async function handleContentClick(e?: MouseEvent) {
  if (!e) {
    await openUrl(articleUrl);
    return;
  }
  const target = e.target as HTMLElement;
  const anchor = target.closest('a');
  if (anchor?.href) {
    e.preventDefault();
    await openUrl(anchor.href);
  }
}
</script>

{#if loadError}
  <div class="error-banner">
    <CircleAlert size={16} />
    <span>Could not extract full content. Showing summary instead.</span>
  </div>
{/if}

<div
  class="summary"
  role="button"
  tabindex="0"
  onclick={handleContentClick}
  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleContentClick(); } }}
>
  {@html displayHtml}
</div>

<style>
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

:global([data-theme="dark"]) .error-banner {
  background-color: #3e1b1b;
  color: #ff9999;
}

.summary {
  line-height: 1.8;
  font-size: 1.15rem;
  font-family: var(--font-body-override, var(--font-body));
  color: var(--color-body, var(--text-primary));
}

.summary :global(p) {
  margin-bottom: 1.5rem;
}

.summary :global(*) {
  /* biome-ignore lint/complexity/noImportantStyles: override RSS feed styles */
  color: inherit !important;
  /* biome-ignore lint/complexity/noImportantStyles: override RSS feed styles */
  background-color: transparent !important;
  /* biome-ignore lint/complexity/noImportantStyles: override RSS feed styles */
  max-width: 100% !important;
}

.summary :global(a) {
  /* biome-ignore lint/complexity/noImportantStyles: override RSS feed link styles */
  color: #4899ec !important;
  text-decoration: none;
  /* biome-ignore lint/complexity/noImportantStyles: override RSS feed pointer */
  cursor: pointer !important;
}

.summary :global(a:hover) {
  text-decoration: underline;
}

.summary :global(img) {
  max-width: 100%;
  height: auto;
  border-radius: 4px;
  /* biome-ignore lint/complexity/noImportantStyles: override RSS feed img bg */
  background-color: initial !important;
}
</style>
