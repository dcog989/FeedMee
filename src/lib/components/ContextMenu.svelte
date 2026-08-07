<script lang="ts">
import type { Snippet } from 'svelte';

let {
  children,
  x = 0,
  y = 0,
  visible = false,
  onClose = () => {},
}: {
  children?: Snippet;
  x: number;
  y: number;
  visible: boolean;
  onClose: () => void;
} = $props();
</script>

<svelte:window onclick={onClose} onkeydown={(e) => { if (e.key === 'Escape') onClose(); }} />

{#if visible}
  <div class="context-menu" style="top: {y}px; left: {x}px" role="menu">
    {@render children?.()}
  </div>
{/if}

<style>
.context-menu {
  position: fixed;
  background: var(--bg-app);
  border: 1px solid var(--border-color);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  border-radius: 6px;
  padding: 4px;
  z-index: 1000;
  min-width: 120px;
}

.context-menu :global(button) {
  display: block;
  width: 100%;
  text-align: left;
  background: none;
  border: none;
  padding: 8px 12px;
  cursor: pointer;
  color: var(--text-primary);
  border-radius: 4px;
  font-size: 0.9rem;
}

.context-menu :global(button:hover) {
  background-color: var(--bg-hover);
}

.context-menu :global(button.danger) {
  color: #e81123;
}

.context-menu :global(button.danger:hover) {
  background-color: #ffeef0;
}
</style>
