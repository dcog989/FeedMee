<script lang="ts">
    import { appState } from "$lib/store";
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

<!-- The app content renders here -->
{@render children()}
