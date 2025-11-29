<script lang="ts">
    import ArticleListPane from "$lib/components/ArticleListPane.svelte";
    import NavPane from "$lib/components/NavPane.svelte";
    import ReadingPane from "$lib/components/ReadingPane.svelte";
</script>

<!-- Note: Global styles and Resets are now in src/app.css -->

<div class="app-container">
    <div class="nav-area">
        <NavPane />
    </div>
    <div class="list-area">
        <ArticleListPane />
    </div>
    <div class="reading-area">
        <ReadingPane />
    </div>
</div>

<style>
    .app-container {
        display: grid;
        height: 100vh;
        width: 100vw;
        background-color: var(--bg-app);

        /* Desktop: Full 3-pane layout */
        grid-template-columns: 280px 320px 1fr;
        grid-template-areas: "nav list reading";
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

    /* Tablet: Collapsed Navigation */
    @media (max-width: 1200px) {
        .app-container {
            grid-template-columns: 200px 300px 1fr;
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

        .nav-area {
            height: 60px;
            border-bottom: 1px solid var(--border-color);
        }
    }
</style>
