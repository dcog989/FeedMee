<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { store } from '$lib/store';
	import type { Folder } from '$lib/types';

	$effect(() => {
		async function loadFolders() {
			try {
				const result = await invoke<Folder[]>('get_folders_with_feeds');
				$store.folders = result || []; // Ensure we always have an array
			} catch (e) {
				console.error(e);
			}
		}
		loadFolders();
	});

	function selectFeed(id: number) {
		$store.selectedFeedId = id;
		$store.selectedArticle = null;
		$store.articles = [];
	}
</script>

<nav class="pane">
	<h2>Feeds</h2>
	<div class="folder-list">
		{#each $store.folders as folder (folder.id)}
			<div class="folder">
				<h3 class="folder-name">{folder.name}</h3>
				<ul class="feed-list">
					{#each folder.feeds as feed (feed.id)}
						<li>
							<button
								class:selected={$store.selectedFeedId === feed.id}
								onclick={() => selectFeed(feed.id)}
							>
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
		background-color: #f3f3f3;
		padding: 1rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
		overflow-y: auto;
		border-right: 1px solid #e0e0e0;
	}

	h2 {
		margin: 0;
		font-size: 1.2rem;
	}

	.folder-name {
		font-size: 0.9rem;
		text-transform: uppercase;
		color: #555;
		margin: 1rem 0 0.5rem;
		font-weight: 600;
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
		border-radius: 4px;
		font-size: 1rem;
	}

	button:hover {
		background-color: #e9e9e9;
	}

	button.selected {
		background-color: #007aff;
		color: white;
		font-weight: 600;
	}

	@media (prefers-color-scheme: dark) {
		.pane {
			background-color: #2a2a2a;
			border-right-color: #3a3a3a;
		}
		.folder-name {
			color: #aaa;
		}
		button:hover {
			background-color: #383838;
		}
		button.selected {
			background-color: #0a84ff;
		}
	}
</style>