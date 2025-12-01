<script lang="ts">
    import { tooltipState } from "$lib/actions/tooltip.svelte";
    // Safety check to keep tooltip on screen would be nice, but simple fixed positioning for MVP
    // We add a transition for ease-in
    import { fade } from "svelte/transition";
</script>

{#if tooltipState.visible}
    <div class="tooltip" style="top: {tooltipState.y}px; left: {tooltipState.x}px;" transition:fade={{ duration: 150 }}>
        {tooltipState.text}
    </div>
{/if}

<style>
    .tooltip {
        position: fixed;
        z-index: 9999;
        background-color: var(--bg-pane);
        border: 1px solid var(--border-color);
        color: var(--text-primary);
        padding: 4px 8px;
        border-radius: 4px;
        font-size: 0.75rem;
        pointer-events: none;
        box-shadow: 0 2px 5px rgba(0, 0, 0, 0.1);
        white-space: nowrap;
    }
</style>
