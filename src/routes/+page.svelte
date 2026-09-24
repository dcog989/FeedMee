<script lang="ts">
import ArticleListPane from "$lib/components/ArticleListPane.svelte";
import NavPane from "$lib/components/NavPane.svelte";
import ReadingPane from "$lib/components/ReadingPane.svelte";
import { appState } from "$lib/store.svelte";

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
      appState.navWidth = Math.max(NAV_MIN_WIDTH, Math.min(NAV_MAX_WIDTH, pendingClientX));
    } else {
      // List width is relative to the nav pane offset.
      appState.listWidth = Math.max(LIST_MIN_WIDTH, Math.min(LIST_MAX_WIDTH, pendingClientX - appState.navWidth));
    }
  });
}

function focusPane(pane: "nav" | "list" | "reading") {
  appState.focusedPane = pane;
}

function onPaneKeyDown(pane: "nav" | "list" | "reading", e: KeyboardEvent) {
  if (e.key === "Enter" || e.key === " ") {
    e.preventDefault();
    focusPane(pane);
  }
}

function onResizerKeyDown(target: "nav" | "list", e: KeyboardEvent) {
  if (e.key === "ArrowLeft") {
    if (target === "nav") {
      appState.navWidth = Math.max(NAV_MIN_WIDTH, appState.navWidth - RESIZE_STEP);
    } else {
      appState.listWidth = Math.max(LIST_MIN_WIDTH, appState.listWidth - RESIZE_STEP);
    }
    e.preventDefault();
  } else if (e.key === "ArrowRight") {
    if (target === "nav") {
      appState.navWidth = Math.min(NAV_MAX_WIDTH, appState.navWidth + RESIZE_STEP);
    } else {
      appState.listWidth = Math.min(LIST_MAX_WIDTH, appState.listWidth + RESIZE_STEP);
    }
    e.preventDefault();
  }
}
</script>

<svelte:window onmouseup={stopResize} onmousemove={onMouseMove} />

<!--
	We inject the dynamic widths as CSS Variables.
	This allows the Media Queries in the <style> block to easily override them
	on mobile without fighting inline style specificity.
-->
<div class="app-container" style="--nav-w: {appState.navWidth}px; --list-w: {appState.listWidth}px;">
  <!-- Navigation Pane -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <section
    class="nav-area"
    aria-label="Navigation"
    class:pane-focused={appState.focusedPane === "nav"}
    onclick={() => focusPane("nav")}
    onkeydown={(e) => onPaneKeyDown("nav", e)}
  >
    <NavPane />
  </section>

  <!-- Resizer Handle 1 (Nav <-> List) -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <hr
    class="resizer nav-resizer"
    aria-orientation="vertical"
    aria-valuenow={appState.navWidth}
    aria-valuemin={NAV_MIN_WIDTH}
    aria-valuemax={NAV_MAX_WIDTH}
    tabindex="0"
    onmousedown={() => startResize("nav")}
    onkeydown={(e) => onResizerKeyDown("nav", e)}
  >

  <!-- Article List Pane -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <section
    class="list-area"
    aria-label="Article List"
    class:pane-focused={appState.focusedPane === "list"}
    onclick={() => focusPane("list")}
    onkeydown={(e) => onPaneKeyDown("list", e)}
  >
    <ArticleListPane />
  </section>

  <!-- Resizer Handle 2 (List <-> Reading) -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <hr
    class="resizer list-resizer"
    aria-orientation="vertical"
    aria-valuenow={appState.listWidth}
    aria-valuemin={LIST_MIN_WIDTH}
    aria-valuemax={LIST_MAX_WIDTH}
    tabindex="0"
    onmousedown={() => startResize("list")}
    onkeydown={(e) => onResizerKeyDown("list", e)}
  >

  <!-- Reading Pane -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <section
    class="reading-area"
    aria-label="Reading"
    class:pane-focused={appState.focusedPane === "reading"}
    onclick={() => focusPane("reading")}
    onkeydown={(e) => onPaneKeyDown("reading", e)}
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
  border-top-color: rgba(236, 72, 153, 0.5);
}

.nav-resizer {
  grid-area: resizer1;
}
.list-resizer {
  grid-area: resizer2;
}

/* Tablet: Collapsed Navigation (Overrides variables) */
@media (max-width: 1200px) {
  .app-container {
    /* Force specific widths, hiding the resizers effectively */
    grid-template-columns: 200px 0px 300px 0px 1fr;
  }

  /* Hide visual handles on smaller screens */
  .resizer {
    display: none;
    pointer-events: none;
  }
}

/* Mobile: Single Pane */
@media (max-width: 768px) {
  .app-container {
    grid-template-columns: 1fr;
    grid-template-rows: auto 1fr;
    grid-template-areas:
      "nav"
      "list";
  }

  .reading-area {
    display: none;
  }
  .resizer {
    display: none;
  }

  .nav-area {
    height: 60px;
    border-bottom: 1px solid var(--border-color);
  }
}
</style>
