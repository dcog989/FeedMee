<script lang="ts">
import { uiStore } from "#lib/store.svelte.js";
import Modal from "./Modal.svelte";
</script>

<Modal
  isOpen={uiStore.modalState.isOpen}
  label={uiStore.modalState.type === "confirm" ? "Confirmation" : "Alert"}
  onclose={() => uiStore.closeModal()}
>
  <h3>
    {uiStore.modalState.type === "confirm" ? "Confirmation" : "Alert"}
  </h3>
  <p>{uiStore.modalState.message}</p>
  <div class="modal-actions">
    {#if uiStore.modalState.type === "confirm"}
      <button type="button" class="secondary" onclick={() => uiStore.closeModal()}>Cancel</button>
    {/if}
    <button
      type="button"
      class={uiStore.modalState.type === "confirm" ? "danger" : "primary"}
      onclick={uiStore.modalState.onConfirm}
    >
      {uiStore.modalState.type === "confirm" ? "Confirm" : "OK"}
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
</style>
