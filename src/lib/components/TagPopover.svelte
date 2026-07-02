<script lang="ts">
import TagManager from './TagManager.svelte';

let {
    articleId,
    x,
    y,
    onClose,
}: {
    articleId: number | null;
    x: number;
    y: number;
    onClose: () => void;
} = $props();
</script>

{#if articleId !== null}
    <button
        type="button"
        class="tag-backdrop"
        aria-label="Close"
        onclick={onClose}
    ></button>
    <div class="tag-popover" style="top: {y}px; left: {x}px">
        <TagManager {articleId} {onClose} />
    </div>
{/if}

<style>
.tag-backdrop {
    position: fixed;
    inset: 0;
    z-index: 999;
    background: transparent;
    border: none;
}

.tag-popover {
    position: fixed;
    z-index: 1000;
}
</style>
