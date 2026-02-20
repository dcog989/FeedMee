<script lang="ts">
    import SettingsModal from '$lib/components/SettingsModal.svelte';
    import TitleBar from '$lib/components/TitleBar.svelte';
    import Tooltip from '$lib/components/Tooltip.svelte';
    import { appState } from '$lib/store.svelte';
    import '../app.css';

    let { children } = $props();

    function disableContextMenu(e: MouseEvent) {
        e.preventDefault();
    }

    $effect(() => {
        const root = document.documentElement;

        if (appState.theme === 'system') {
            const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
            root.setAttribute('data-theme', prefersDark ? 'dark' : 'light');
        } else {
            root.setAttribute('data-theme', appState.theme);
        }
    });
</script>

<div class="layout-wrapper" role="application" oncontextmenu={disableContextMenu}>
    <TitleBar />
    <div class="content-wrapper">
        {@render children()}
    </div>
    <Tooltip />

    {#if appState.showSettings}
        <SettingsModal />
    {/if}

    {#if appState.modalState.isOpen}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="modal-overlay" onclick={() => appState.closeModal()}>
            <div class="modal" onclick={(e) => e.stopPropagation()}>
                <h3>{appState.modalState.type === 'confirm' ? 'Confirmation' : 'Alert'}</h3>
                <p>{appState.modalState.message}</p>
                <div class="modal-actions">
                    {#if appState.modalState.type === 'confirm'}
                        <button class="secondary" onclick={() => appState.closeModal()}
                            >Cancel</button>
                    {/if}
                    <button
                        class={appState.modalState.type === 'confirm' ? 'danger' : 'primary'}
                        onclick={appState.modalState.onConfirm}>
                        {appState.modalState.type === 'confirm' ? 'Confirm' : 'OK'}
                    </button>
                </div>
            </div>
        </div>
    {/if}
</div>

<style>
    .layout-wrapper {
        display: flex;
        flex-direction: column;
        height: 100vh;
        width: 100vw;
        overflow: hidden;
    }

    .content-wrapper {
        flex: 1;
        overflow: hidden;
        position: relative;
    }

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
