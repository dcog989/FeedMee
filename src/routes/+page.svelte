<script lang="ts">
    import ArticleListPane from '$lib/components/ArticleListPane.svelte';
    import NavPane from '$lib/components/NavPane.svelte';
    import ReadingPane from '$lib/components/ReadingPane.svelte';
    import { appState } from '$lib/store.svelte';

    // Resizing Logic
    let isResizing = $state<'nav' | 'list' | null>(null);

    function startResize(target: 'nav' | 'list') {
        isResizing = target;
        // Add global cursor style and disable text selection while dragging
        document.body.style.cursor = 'col-resize';
        document.body.style.userSelect = 'none';
    }

    function stopResize() {
        if (isResizing) {
            isResizing = null;
            document.body.style.cursor = '';
            document.body.style.userSelect = '';
        }
    }

    function onMouseMove(e: MouseEvent) {
        if (!isResizing) return;

        if (isResizing === 'nav') {
            // Min width 150px, Max width 500px
            const newWidth = Math.max(150, Math.min(500, e.clientX));
            appState.navWidth = newWidth;
        } else if (isResizing === 'list') {
            // Calculate width based on Nav width offset
            // Min 200px, Max 600px
            const newWidth = Math.max(200, Math.min(600, e.clientX - appState.navWidth));
            appState.listWidth = newWidth;
        }
    }

    function focusPane(pane: 'nav' | 'list' | 'reading') {
        appState.focusedPane = pane;
    }
</script>

<svelte:window onmouseup={stopResize} onmousemove={onMouseMove} />

<!--
	We inject the dynamic widths as CSS Variables.
	This allows the Media Queries in the <style> block to easily override them
	on mobile without fighting inline style specificity.
-->
<div
    class="app-container"
    style="--nav-w: {appState.navWidth}px; --list-w: {appState.listWidth}px;">
    <!-- Navigation Pane -->
    <div
        class="nav-area"
        class:pane-focused={appState.focusedPane === 'nav'}
        onclick={() => focusPane('nav')}>
        <NavPane />
    </div>

    <!-- Resizer Handle 1 (Nav <-> List) -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="resizer nav-resizer" onmousedown={() => startResize('nav')}></div>

    <!-- Article List Pane -->
    <div
        class="list-area"
        class:pane-focused={appState.focusedPane === 'list'}
        onclick={() => focusPane('list')}>
        <ArticleListPane />
    </div>

    <!-- Resizer Handle 2 (List <-> Reading) -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="resizer list-resizer" onmousedown={() => startResize('list')}></div>

    <!-- Reading Pane -->
    <div
        class="reading-area"
        class:pane-focused={appState.focusedPane === 'reading'}
        onclick={() => focusPane('reading')}>
        <ReadingPane />
    </div>
</div>

<style>
    .app-container {
        display: grid;
        height: 100%; /* Changed from 100vh */
        width: 100vw;
        background-color: var(--bg-app);

        /*
		   Desktop: Use dynamic variables with Fallbacks.
		   The fallbacks (280px, 320px) ensure the layout works immediately on load,
		   fixing the issue where panes were not resizable until window resize.
		*/
        grid-template-columns: var(--nav-w, 280px) 4px var(--list-w, 320px) 4px 1fr;
        grid-template-areas: 'nav resizer1 list resizer2 reading';
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
                'nav'
                'list';
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
