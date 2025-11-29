<script lang="ts">
    import { appState } from "$lib/store";
</script>

<nav class="pane">
    <div class="header">
        <h2>Feeds</h2>
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
        padding: 1rem;
    }

    h2 {
        margin: 0;
        font-size: 1.2rem;
        color: var(--text-primary);
    }

    .folder-list {
        flex: 1;
        overflow-y: auto;
        padding: 0 1rem 1rem;
    }

    .folder-name {
        font-size: 0.8rem;
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
