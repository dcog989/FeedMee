<script lang="ts">
    import TitleBar from "$lib/components/TitleBar.svelte";
    import { appState } from "$lib/store.svelte";
    import "../app.css";

    let { children } = $props();

    $effect(() => {
        const root = document.documentElement;

        if (appState.theme === "system") {
            const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
            root.setAttribute("data-theme", prefersDark ? "dark" : "light");
        } else {
            root.setAttribute("data-theme", appState.theme);
        }
    });
</script>

<div class="layout-wrapper">
    <TitleBar />
    <div class="content-wrapper">
        {@render children()}
    </div>
</div>

<style>
    .layout-wrapper {
        display: flex;
        flex-direction: column;
        height: 100vh;
        width: 100vw;
        overflow: hidden;
    }

    .content-wrapper {
        flex: 1;
        overflow: hidden;
        position: relative;
    }
</style>
