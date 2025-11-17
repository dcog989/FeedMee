<script lang="ts">
	import ArticleListPane from '$lib/components/ArticleListPane.svelte';
	import NavPane from '$lib/components/NavPane.svelte';
	import ReadingPane from '$lib/components/ReadingPane.svelte';
</script>

<div class="app-container">
	<NavPane />
	<ArticleListPane />
	<ReadingPane />
</div>

<style>
	:global(body, html) {
		margin: 0;
		padding: 0;
		height: 100%;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial,
			sans-serif, 'Apple Color Emoji', 'Segoe UI Emoji';
		overflow: hidden;
		background-color: #fff;
		color: #111;
	}

	@media (prefers-color-scheme: dark) {
		:global(body, html) {
			background-color: #212121;
			color: #f1f1f1;
		}
	}

	.app-container {
		display: grid;
		grid-template-columns: 280px 320px 1fr;
		height: 100vh;
		width: 100vw;
	}
</style>