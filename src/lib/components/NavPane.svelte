<script lang="ts">
    import { appState } from "$lib/store.svelte";
</script>

<nav class="pane">
    <div class="header">
        <div class="brand">
            <h2>FeedMee</h2>
        </div>
        <button class="action-btn" onclick={() => appState.importOpml()} title="Import OPML"> Import </button>
    </div>

    <div class="folder-list">
        {#each appState.folders as folder (folder.id)}
            <div class="folder">
                <h3 class="folder-name">{folder.name}</h3>
                <ul class="feed-list">
                    {#each folder.feeds as feed (feed.id)}
                        <li>
                            <button class:selected={appState.selectedFeedId === feed.id} onclick={() => appState.selectFeed(feed.id)}>
                                {feed.name}
                            </button>
                        </li>
                    {/each}
                </ul>
            </div>
        {/each}
    </div>
</nav>

<style>
    .pane {
        background-color: var(--bg-pane);
        border-right: 1px solid var(--border-color);
        height: 100%;
        display: flex;
        flex-direction: column;
        box-sizing: border-box;
    }

    .header {
        padding: 1.5rem 1rem 1rem;
        display: flex;
        flex-direction: column;
        gap: 1rem;
        border-bottom: 1px solid transparent;
    }

    .brand {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .logo {
        width: 32px;
        height: 32px;
    }

    h2 {
        margin: 0;
        font-size: 1.4rem;
        font-weight: 800;
        color: var(--text-primary);
        letter-spacing: -0.5px;
    }

    .action-btn {
        width: 100%;
        padding: 0.5rem;
        font-size: 0.9rem;
        background-color: var(--bg-content);
        border: 1px solid var(--border-color);
        color: var(--text-primary);
        border-radius: 6px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s;
    }

    .action-btn:hover {
        border-color: var(--text-secondary);
        background-color: var(--bg-hover);
    }

    .folder-list {
        flex: 1;
        overflow-y: auto;
        padding: 1rem;
    }

    .folder-name {
        font-size: 0.75rem;
        text-transform: uppercase;
        color: var(--text-secondary);
        margin: 1rem 0 0.5rem;
        font-weight: 700;
        letter-spacing: 0.5px;
    }

    .feed-list {
        list-style: none;
        padding: 0;
        margin: 0;
    }

    button {
        width: 100%;
        padding: 0.5rem 0.75rem;
        background: transparent;
        border: none;
        text-align: left;
        cursor: pointer;
        border-radius: 6px;
        font-size: 0.95rem;
        color: var(--text-primary);
        transition: background-color 0.1s;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    button:hover {
        background-color: var(--bg-hover);
    }

    button.selected {
        background-color: var(--bg-selected);
        color: var(--text-inverse);
        font-weight: 500;
    }
</style>
