<script lang="ts">
import { applyThemeToDocument, settingsStore } from "$lib/store.svelte";

$effect(() => {
  applyThemeToDocument(settingsStore.theme);

  if (settingsStore.theme !== "system") return;

  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  const apply = () => applyThemeToDocument("system");
  mq.addEventListener("change", apply);
  return () => mq.removeEventListener("change", apply);
});
</script>
