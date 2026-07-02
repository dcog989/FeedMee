<script lang="ts">
import { uiStore } from '$lib/store.svelte';
</script>

{#if uiStore.modalState.isOpen}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- biome-ignore lint/a11y/noStaticElementInteractions: backdrop is decorative, can't use button because it wraps modal with buttons -->
    <div
        class="modal-overlay"
        role="presentation"
        onclick={(e) => { if (e.target === e.currentTarget) uiStore.closeModal(); }}
    >
        <div class="modal">
            <h3>
                {uiStore.modalState.type === "confirm"
                    ? "Confirmation"
                    : "Alert"}
            </h3>
            <p>{uiStore.modalState.message}</p>
            <div class="modal-actions">
                {#if uiStore.modalState.type === "confirm"}
                    <button
                        type="button"
                        class="secondary"
                        onclick={() => uiStore.closeModal()}
                    >
                        Cancel
                    </button>
                {/if}
                <button
                    type="button"
                    class={uiStore.modalState.type === "confirm"
                        ? "danger"
                        : "primary"}
                    onclick={uiStore.modalState.onConfirm}
                >
                    {uiStore.modalState.type === "confirm"
                        ? "Confirm"
                        : "OK"}
                </button>
            </div>
        </div>
    </div>
{/if}

<style>
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 10000;
    backdrop-filter: blur(2px);
}

.modal {
    background: var(--bg-pane);
    padding: 1.5rem;
    border-radius: 8px;
    width: 350px;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
}

.modal h3 {
    margin-top: 0;
    font-size: 1.1rem;
}

.modal p {
    margin: 1rem 0 1.5rem 0;
    color: var(--text-secondary);
}

.modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
}

button {
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 500;
    border: none;
}

button.secondary {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-primary);
}

button.secondary:hover {
    background: var(--bg-hover);
}

button.danger {
    background: #e81123;
    color: white;
}

button.primary {
    background: var(--bg-selected);
    color: white;
}

button.danger:hover,
button.primary:hover {
    opacity: 0.9;
}
</style>
