<script lang="ts">
	import { store } from '$lib/store';
</script>

<main class="pane">
	{#if $store.selectedArticle}
		<article class="article-content">
			<h1>{$store.selectedArticle.title}</h1>
			<div class="meta">
				<span>By {$store.selectedArticle.author}</span>
				<span>{new Date($store.selectedArticle.timestamp * 1000).toLocaleString()}</span>
			</div>
			<div class="summary">
				{@html $store.selectedArticle.summary}
			</div>
		</article>
	{:else}
		<div class="empty-state">
			<p>Select an article to start reading</p>
		</div>
	{/if}
</main>

<style>
	.pane {
		overflow-y: auto;
		padding: 2rem;
	}

	.article-content {
		max-width: 800px;
		margin: 0 auto;
	}

	h1 {
		font-size: 2rem;
		margin-bottom: 0.5rem;
	}

	.meta {
		color: #666;
		margin-bottom: 2rem;
		font-size: 0.9rem;
		display: flex;
		gap: 1rem;
		border-bottom: 1px solid #eee;
		padding-bottom: 1rem;
	}

	.summary {
		line-height: 1.7;
		font-size: 1.1rem;
	}

	.empty-state {
		display: flex;
		justify-content: center;
		align-items: center;
		height: 100%;
		color: #888;
		font-size: 1.2rem;
	}
	@media (prefers-color-scheme: dark) {
		.meta {
			color: #aaa;
			border-bottom-color: #3a3a3a;
		}
		.empty-state {
			color: #888;
		}
	}
</style>