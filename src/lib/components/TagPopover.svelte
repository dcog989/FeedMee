<script lang="ts">
import { untrack } from "svelte";
import TagManager from "./TagManager.svelte";

let {
  articleId,
  x,
  y,
  onClose,
}: {
  articleId: number | null;
  x: number;
  y: number;
  onClose: () => void;
} = $props();

let popoverEl = $state<HTMLDivElement>();
let left = $state(untrack(() => x));
let top = $state(untrack(() => y));

// Keep the popover on screen: a tag button near the bottom edge would open a
// panel that runs past the window and cannot be reached (same clamp as the
// context menu).
$effect(() => {
  if (!popoverEl) return;
  const rect = popoverEl.getBoundingClientRect();
  left = Math.max(8, Math.min(x, window.innerWidth - rect.width - 8));
  top = Math.max(8, Math.min(y, window.innerHeight - rect.height - 8));
});
</script>

{#if articleId !== null}
  <button type="button" class="tag-backdrop" aria-label="Close" onclick={onClose}></button>
  <div class="tag-popover" bind:this={popoverEl} style="top: {top}px; left: {left}px">
    <TagManager {articleId} {onClose} />
  </div>
{/if}

<style>
.tag-backdrop {
  position: fixed;
  inset: 0;
  z-index: 999;
  background: transparent;
  border: none;
}

.tag-popover {
  position: fixed;
  z-index: 1000;
}
</style>
