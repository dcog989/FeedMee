<script lang="ts">
import ArticleListPane from "#lib/components/ArticleListPane.svelte";
import NavPane from "#lib/components/NavPane.svelte";
import ReadingPane from "#lib/components/ReadingPane.svelte";
import { appState, uiStore } from "#lib/store.svelte.js";

// Resizing Logic
const NAV_MIN_WIDTH = 150;
const NAV_MAX_WIDTH = 500;
const LIST_MIN_WIDTH = 200;
const LIST_MAX_WIDTH = 900;
const RESIZE_STEP = 10;

let isResizing = $state<"nav" | "list" | null>(null);
let resizeFrame: number | null = null;
let pendingClientX = 0;

function startResize(target: "nav" | "list") {
  isResizing = target;
  // Add global cursor style and disable text selection while dragging
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
}

function stopResize() {
  if (isResizing) {
    isResizing = null;
    if (resizeFrame !== null) {
      cancelAnimationFrame(resizeFrame);
      resizeFrame = null;
    }
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    appState.persistLayoutSettings();
  }
}

function onMouseMove(e: MouseEvent) {
  if (!isResizing) return;

  pendingClientX = e.clientX;
  if (resizeFrame !== null) return;

  const target = isResizing;
  resizeFrame = requestAnimationFrame(() => {
    resizeFrame = null;
    if (target === "nav") {
      uiStore.navWidth = Math.max(NAV_MIN_WIDTH, Math.min(NAV_MAX_WIDTH, pendingClientX));
    } else {
      // List width is relative to the nav pane offset.
      uiStore.listWidth = Math.max(LIST_MIN_WIDTH, Math.min(LIST_MAX_WIDTH, pendingClientX - uiStore.navWidth));
    }
  });
}

function focusPane(pane: "nav" | "list" | "reading") {
  appState.focusedPane = pane;
}

// Action instead of onclick so the panes stay non-interactive for a11y lint:
// the click only marks which pane the arrow keys control.
function focusPaneClick(node: HTMLElement, pane: "nav" | "list" | "reading") {
  const onClick = () => focusPane(pane);
  node.addEventListener("click", onClick);
  return {
    destroy() {
      node.removeEventListener("click", onClick);
    },
  };
}

function onResizerKeyDown(target: "nav" | "list", e: KeyboardEvent) {
  if (e.key === "ArrowLeft") {
    if (target === "nav") {
      uiStore.navWidth = Math.max(NAV_MIN_WIDTH, uiStore.navWidth - RESIZE_STEP);
    } else {
      uiStore.listWidth = Math.max(LIST_MIN_WIDTH, uiStore.listWidth - RESIZE_STEP);
    }
    e.preventDefault();
  } else if (e.key === "ArrowRight") {
    if (target === "nav") {
      uiStore.navWidth = Math.min(NAV_MAX_WIDTH, uiStore.navWidth + RESIZE_STEP);
    } else {
      uiStore.listWidth = Math.min(LIST_MAX_WIDTH, uiStore.listWidth + RESIZE_STEP);
    }
    e.preventDefault();
  } else {
    return;
  }
  appState.persistLayoutSettings();
}
</script>

<svelte:window onmouseup={stopResize} onmousemove={onMouseMove} />

<!--
	We inject the dynamic widths as CSS Variables.
	This allows the Media Queries in the <style> block to easily override them
	on mobile without fighting inline style specificity.
-->
<div class="app-container" style="--nav-w: {uiStore.navWidth}px; --list-w: {uiStore.listWidth}px;">
  <!-- Navigation Pane -->
  <section
    class="nav-area"
    aria-label="Navigation"
    class:pane-focused={appState.focusedPane === "nav"}
    use:focusPaneClick={"nav"}
  >
    <NavPane />
  </section>

  <!-- Resizer Handle 1 (Nav <-> List) -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <hr
    class="resizer nav-resizer"
    aria-orientation="vertical"
    aria-valuenow={uiStore.navWidth}
    aria-valuemin={NAV_MIN_WIDTH}
    aria-valuemax={NAV_MAX_WIDTH}
    tabindex="0"
    onmousedown={() => startResize("nav")}
    onkeydown={(e) => onResizerKeyDown("nav", e)}
  >

  <!-- Article List Pane -->
  <section
    class="list-area"
    aria-label="Article List"
    class:pane-focused={appState.focusedPane === "list"}
    use:focusPaneClick={"list"}
  >
    <ArticleListPane />
  </section>

  <!-- Resizer Handle 2 (List <-> Reading) -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <hr
    class="resizer list-resizer"
    aria-orientation="vertical"
    aria-valuenow={uiStore.listWidth}
    aria-valuemin={LIST_MIN_WIDTH}
    aria-valuemax={LIST_MAX_WIDTH}
    tabindex="0"
    onmousedown={() => startResize("list")}
    onkeydown={(e) => onResizerKeyDown("list", e)}
  >

  <!-- Reading Pane -->
  <section
    class="reading-area"
    aria-label="Reading"
    class:pane-focused={appState.focusedPane === "reading"}
    use:focusPaneClick={"reading"}
  >
    <ReadingPane />
  </section>
</div>

<style>
.app-container {
  display: grid;
  height: 100%; /* Changed from 100vh */
  width: 100vw;
  background-color: var(--bg-app);

  /*
		   Define the default column layout in CSS so it applies immediately on
		   first paint, before JS has mounted and set the inline --nav-w/--list-w
		   variables. Without this, Vite dev's first-run timing gap leaves the
		   grid with no column definition and the panes stack vertically.
		*/
  --nav-w: 280px;
  --list-w: 320px;
  grid-template-columns: var(--nav-w, 280px) 4px var(--list-w, 320px) 4px 1fr;
  grid-template-areas: "nav resizer1 list resizer2 reading";
}

.nav-area {
  grid-area: nav;
  overflow: hidden;
}
.list-area {
  grid-area: list;
  overflow: hidden;
}
.reading-area {
  grid-area: reading;
  overflow: hidden;
}

.resizer {
  border: none;
  margin: 0;
  height: 100%;
  width: 100%;
  background-color: transparent;
  cursor: col-resize;
  z-index: 10;
  transition: background-color 0.2s;
}

.resizer:hover,
.resizer:active {
  background-color: var(--bg-selected);
}

.nav-area,
.list-area,
.reading-area {
  border-top: 2px solid transparent;
  box-sizing: border-box;
}

.pane-focused {
  border-top-color: var(--bg-selected-muted);
}

.nav-resizer {
  grid-area: resizer1;
}
.list-resizer {
  grid-area: resizer2;
}
</style>
