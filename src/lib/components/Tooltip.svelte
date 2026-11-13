<script lang="ts">
import { fade } from "svelte/transition";
import { tooltipState } from "#lib/actions/tooltip.svelte.js";

let el = $state<HTMLDivElement>();
let left = $state(0);
let top = $state(0);

// The cursor offset pushes the hint past the right and bottom edges, so clamp
// it like the context menu and tag popover do.
$effect(() => {
  if (!el) return;
  const rect = el.getBoundingClientRect();
  left = Math.max(8, Math.min(tooltipState.x, window.innerWidth - rect.width - 8));
  top = Math.max(8, Math.min(tooltipState.y, window.innerHeight - rect.height - 8));
});
</script>

{#if tooltipState.visible}
  <div class="tooltip" bind:this={el} style="top: {top}px; left: {left}px;" transition:fade={{ duration: 150 }}>
    {tooltipState.text}
  </div>
{/if}

<style>
.tooltip {
  position: fixed;
  z-index: 9999;
  background-color: var(--bg-pane);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 0.75rem;
  pointer-events: none;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.1);
  white-space: nowrap;
}
</style>
