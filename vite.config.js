import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig({
  plugins: await sveltekit(),
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    chunkSizeWarningLimit: 1000,
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [
            { name: "svelte", test: /[\\/]node_modules[\\/]svelte[\\/]/ },
            { name: "tauri", test: /[\\/]node_modules[\\/]@tauri-apps[\\/]/ },
          ],
        },
      },
    },
  },
  // Force dep pre-bundling before Tauri opens the webview on cold start.
  // Prevents stylesheets arriving late and layout collapsing on first `bun run dev`.
  // Keep in sync with bare-specifier imports on the initial-render module graph.
  //
  // Svelte component libraries (e.g. @lucide/svelte) MUST be pre-bundled here. When
  // excluded, Vite's dep scanner crawls their .svelte source and loads virtual
  // `?svelte&type=style&lang.css` modules before the parent component metadata is
  // set, so some components receive their raw Svelte source as CSS. That drops
  // scoped rules (e.g. `.app-container { display: grid }`) and the panes stack.
  optimizeDeps: {
    include: [
      "dompurify",
      "@lucide/svelte",
      "svelte/animate",
      "svelte/transition",
      "@tauri-apps/api/core",
      "@tauri-apps/plugin-opener",
      "@tauri-apps/plugin-dialog",
    ],
  },
});
