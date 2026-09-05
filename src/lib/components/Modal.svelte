<script lang="ts">
import type { Snippet } from "svelte";

let {
  isOpen = false,
  onclose = () => {},
  width = "auto",
  zindex = 10000,
  class: className = "",
  children,
}: {
  isOpen?: boolean;
  onclose?: () => void;
  width?: string;
  zindex?: number;
  class?: string;
  children: Snippet;
} = $props();

function onOverlayClick(e: MouseEvent) {
  if (e.target === e.currentTarget) onclose();
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape" && !e.defaultPrevented) onclose();
}
</script>

<svelte:window onkeydown={onKeyDown} />

{#if isOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- biome-ignore lint/a11y/noStaticElementInteractions: backdrop is decorative, can't use button because it wraps modal content -->
  <div class="modal-overlay" role="presentation" onclick={onOverlayClick} style="z-index: {zindex}">
    <div class="modal {className}" role="dialog" aria-modal="true" tabindex="-1" style="width: {width}">
      {@render children()}
    </div>
  </div>
{/if}

<style>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(2px);
}

.modal {
  background: var(--bg-pane);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.25);
  padding: 1.5rem;
  max-width: calc(100vw - 2rem);
  max-height: 80vh;
  overflow: auto;
  color: var(--text-primary);
}
</style>
