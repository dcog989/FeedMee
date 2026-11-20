<script module lang="ts">
// Open dialogs, oldest first. Only the newest one answers Escape, so a confirm
// or the shortcuts sheet does not also close the window underneath it.
const openDialogs: symbol[] = [];

// Restoring focus after a mouse-opened dialog would paint a keyboard focus ring
// on the trigger once it is closed with Escape, so track how it was opened.
let pointerInitiated = false;
if (typeof window !== "undefined") {
  window.addEventListener("keydown", () => (pointerInitiated = false), true);
  window.addEventListener("pointerdown", () => (pointerInitiated = true), true);
}
</script>

<script lang="ts">
import { type Snippet, untrack } from "svelte";

let {
  isOpen = false,
  onclose = () => {},
  width = "auto",
  zindex = 10000,
  label = "",
  class: className = "",
  children,
}: {
  isOpen?: boolean;
  onclose?: () => void;
  width?: string;
  zindex?: number;
  label?: string;
  class?: string;
  children: Snippet;
} = $props();

const dialogId = Symbol();
let dialogEl = $state<HTMLDivElement>();

$effect(() => {
  if (!isOpen) return;
  openDialogs.push(dialogId);
  return () => {
    const index = openDialogs.lastIndexOf(dialogId);
    if (index !== -1) openDialogs.splice(index, 1);
  };
});

// Give the dialog focus, so Tab enters it and Enter cannot re-fire the button
// that opened it, then put focus back where it was. A dialog opened by pointer
// does not restore focus: the trigger would show a keyboard focus ring.
$effect(() => {
  if (!isOpen) return;
  const previous = document.activeElement as HTMLElement | null;
  const restoreFocus = !pointerInitiated;
  untrack(() => dialogEl)?.focus();
  return () => {
    if (restoreFocus) previous?.focus();
  };
});

function onOverlayClick(e: MouseEvent) {
  if (e.target === e.currentTarget) onclose();
}

const FOCUSABLE = "button, input, select, textarea, a[href], [tabindex]:not([tabindex='-1'])";

function onKeyDown(e: KeyboardEvent) {
  if (e.defaultPrevented) return;
  if (e.key === "Escape") {
    if (openDialogs.at(-1) === dialogId) onclose();
    return;
  }
  if (e.key !== "Tab" || openDialogs.at(-1) !== dialogId) return;

  // Keep Tab inside: otherwise it walks onto the panes hidden behind the overlay.
  const items = [...(dialogEl?.querySelectorAll<HTMLElement>(FOCUSABLE) ?? [])].filter(
    (el) => !el.hasAttribute("disabled"),
  );
  if (items.length === 0) {
    e.preventDefault();
    dialogEl?.focus();
    return;
  }
  const first = items[0];
  const last = items[items.length - 1];
  const active = document.activeElement as HTMLElement | null;
  if (!active || !dialogEl?.contains(active) || active === dialogEl) {
    e.preventDefault();
    (e.shiftKey ? last : first).focus();
  } else if (e.shiftKey && active === first) {
    e.preventDefault();
    last.focus();
  } else if (!e.shiftKey && active === last) {
    e.preventDefault();
    first.focus();
  }
}
</script>

<svelte:window onkeydown={onKeyDown} />

{#if isOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- biome-ignore lint/a11y/noStaticElementInteractions: backdrop is decorative, can't use button because it wraps modal content -->
  <div class="modal-overlay" role="presentation" onclick={onOverlayClick} style="z-index: {zindex}">
    <div
      class="modal {className}"
      role="dialog"
      aria-modal="true"
      aria-label={label || undefined}
      tabindex="-1"
      style="width: {width}"
      bind:this={dialogEl}
    >
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

.modal:focus-visible {
  outline: none;
}
</style>
