<script lang="ts">
import { appState } from '$lib/store.svelte';

$effect(() => {
    const s = appState.settings;
    const root = document.documentElement;

    for (const [prop, value] of [
        ['--font-title', s.article_title_font],
        ['--font-body-override', s.article_body_font],
        ['--color-title', s.article_title_color],
        ['--color-body', s.article_body_color],
        ['--bg-article', s.article_bg_color],
    ] as const) {
        if (value) {
            root.style.setProperty(prop, value);
        } else {
            root.style.removeProperty(prop);
        }
    }
});
</script>
