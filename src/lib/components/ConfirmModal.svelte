<script lang="ts">
import { uiStore } from '$lib/store.svelte';
import Modal from './Modal.svelte';
</script>

<Modal isOpen={uiStore.modalState.isOpen} onclose={() => uiStore.closeModal()}>
  <h3>
    {uiStore.modalState.type === "confirm"
            ? "Confirmation"
            : "Alert"}
  </h3>
  <p>{uiStore.modalState.message}</p>
  <div class="modal-actions">
    {#if uiStore.modalState.type === "confirm"}
      <button type="button" class="secondary" onclick={() => uiStore.closeModal()}>Cancel</button>
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
</Modal>

<style>
h3 {
  margin: 0;
  font-size: 1.1rem;
}

p {
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
