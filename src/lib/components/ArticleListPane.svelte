<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { store } from '$lib/store';
	import type { Article } from '$lib/types';

	$effect(() => {
		async function loadArticles() {
			if ($store.selectedFeedId !== null) {
				try {
					const result = await invoke<Article[]>('get_articles_for_feed', {
						feedId: $store.selectedFeedId
					});
					$store.articles = result || []; // Ensure we always have an array
				} catch (e) {
					console.error(e);
					$store.articles = [];
				}
			} else {
				$store.articles = [];
			}
		}

		loadArticles();
	});
</script>

<section class="pane">
	{#if $store.articles.length > 0}
		<ul class="article-list">
			{#each $store.articles as article (article.id)}
				<li>
					<button
						class:selected={$store.selectedArticle?.id === article.id}
						onclick={() => ($store.selectedArticle = article)}
					>
						<span class="title">{article.title}</span>
						<span class="author">{article.author}</span>
					</button>
				</li>
			{/each}
		</ul>
	{:else if $store.selectedFeedId}
		<p class="empty-state">No articles in this feed.</p>
	{:else}
		<p class="empty-state">Select a feed to see articles.</p>
	{/if}
</section>

<style>
	.pane {
		overflow-y: auto;
		border-right: 1px solid #e0e0e0;
	}
	.article-list {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	button {
		display: block;
		width: 100%;
		padding: 1rem;
		text-align: left;
		border: none;
		border-bottom: 1px solid #e0e0e0;
		background: transparent;
		cursor: pointer;
		font-size: 1rem;
	}

	button:hover {
		background-color: #f5f5f5;
	}

	button.selected {
		background-color: #eef5ff;
	}

	.title {
		display: block;
		font-weight: 600;
		margin-bottom: 0.25rem;
	}

	.author {
		font-size: 0.85rem;
		color: #666;
	}

	.empty-state {
		padding: 2rem;
		text-align: center;
		color: #888;
	}

	@media (prefers-color-scheme: dark) {
		.pane {
			border-right-color: #3a3a3a;
		}
		button {
			border-bottom-color: #3a3a3a;
		}
		button:hover {
			background-color: #333;
		}
		button.selected {
			background-color: #2a3a50;
		}
		.author {
			color: #aaa;
		}
		.empty-state {
			color: #888;
		}
	}
</style>