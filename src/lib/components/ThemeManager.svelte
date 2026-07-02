<script lang="ts">
import { settingsStore } from '$lib/store.svelte';

$effect(() => {
    const root = document.documentElement;

    if (settingsStore.theme !== 'system') {
        root.setAttribute('data-theme', settingsStore.theme);
        return;
    }

    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const apply = () => root.setAttribute('data-theme', mq.matches ? 'dark' : 'light');
    apply();
    mq.addEventListener('change', apply);
    return () => mq.removeEventListener('change', apply);
});
</script>
