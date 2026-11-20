<script lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { uiStore } from "#lib/store.svelte.js";
import type { AppSettings } from "#lib/types.js";

let { settings, onchange }: { settings: AppSettings; onchange: (next: AppSettings) => void } = $props();

function update(patch: Partial<AppSettings>) {
  onchange({ ...settings, ...patch });
}

// <input type="color"> only accepts #rrggbb; expand shorthand hex and drop alpha.
const HEX_COLOR = /^#([0-9a-f]{3,8})$/i;

function swatchValue(value: string): string {
  const match = HEX_COLOR.exec(value.trim());
  if (!match) return "#000000";
  const hex = match[1];
  if (hex.length === 3 || hex.length === 4) {
    return `#${hex[0]}${hex[0]}${hex[1]}${hex[1]}${hex[2]}${hex[2]}`;
  }
  if (hex.length === 6 || hex.length === 8) return `#${hex.slice(0, 6)}`;
  return "#000000";
}

async function pickFont(target: "title" | "body") {
  try {
    const font = await invoke<string>("pick_system_font");
    if (target === "title") update({ article_title_font: font });
    else update({ article_body_font: font });
  } catch (e) {
    const msg = String(e);
    if (msg !== "Font selection cancelled") {
      uiStore.alert(msg);
    }
  }
}
</script>

<h4 class="section-label">Typography</h4>

<div class="form-group">
  <label for="title-font">Article Title Font</label>
  <div class="font-input-wrap">
    <input
      type="text"
      id="title-font"
      value={settings.article_title_font}
      placeholder="Default (Serif)"
      oninput={(e) => update({ article_title_font: e.currentTarget.value })}
    >
    <button type="button" class="pick-btn" onclick={() => pickFont("title")}>Pick…</button>
  </div>
</div>

<div class="form-group">
  <label for="body-font">Article Body Font</label>
  <div class="font-input-wrap">
    <input
      type="text"
      id="body-font"
      value={settings.article_body_font}
      placeholder="Default (Sans)"
      oninput={(e) => update({ article_body_font: e.currentTarget.value })}
    >
    <button type="button" class="pick-btn" onclick={() => pickFont("body")}>Pick…</button>
  </div>
</div>

<hr>

<h4 class="section-label">Colors</h4>

<div class="form-group">
  <label for="title-color">Article Title FG</label>
  <div class="color-input-wrap">
    <input
      type="color"
      id="title-color"
      value={swatchValue(settings.article_title_color)}
      oninput={(e) => update({ article_title_color: e.currentTarget.value })}
    >
    <input
      type="text"
      value={settings.article_title_color}
      oninput={(e) => update({ article_title_color: e.currentTarget.value })}
    >
  </div>
</div>

<div class="form-group">
  <label for="body-color">Article Body FG</label>
  <div class="color-input-wrap">
    <input
      type="color"
      id="body-color"
      value={swatchValue(settings.article_body_color)}
      oninput={(e) => update({ article_body_color: e.currentTarget.value })}
    >
    <input
      type="text"
      value={settings.article_body_color}
      oninput={(e) => update({ article_body_color: e.currentTarget.value })}
    >
  </div>
</div>

<div class="form-group">
  <label for="bg-color">Article Body BG</label>
  <div class="color-input-wrap">
    <input
      type="color"
      id="bg-color"
      value={swatchValue(settings.article_bg_color)}
      oninput={(e) => update({ article_bg_color: e.currentTarget.value })}
    >
    <input
      type="text"
      value={settings.article_bg_color}
      oninput={(e) => update({ article_bg_color: e.currentTarget.value })}
    >
  </div>
</div>

<style>
.font-input-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.pick-btn {
  flex: 0 0 auto;
  padding: 6px 12px;
  border: 1px solid var(--border-color);
  background: var(--bg-app);
  color: var(--text-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8rem;
}

.pick-btn:hover {
  background: var(--bg-hover);
}

.color-input-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.color-input-wrap input[type="color"] {
  width: 32px;
  height: 32px;
  padding: 2px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-app);
  cursor: pointer;
  flex: 0 0 auto;
}

.color-input-wrap input[type="text"] {
  flex: 1;
  padding: 8px;
  border: 1px solid var(--border-color);
  background: var(--bg-app);
  color: var(--text-primary);
  border-radius: 4px;
  box-sizing: border-box;
  font-family: var(--font-mono);
  font-size: 0.8rem;
}
</style>
