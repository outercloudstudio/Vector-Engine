<script lang="ts">
	import { scenes } from '../stores/projectStore'
</script>

<main>
	{#each Object.keys($scenes) as scene}
		<div class="clip">
			<p>{scene}</p>
		</div>
	{/each}
</main>

<style>
	main {
		background-color: var(--main);

		grid-column-start: 1;
		grid-column-end: 2;
		grid-row-start: 1;
		grid-row-end: 2;

		border-right: solid 1px var(--secondary);

		display: flex;
		flex-wrap: wrap;
		justify-content: space-around;
		align-items: start;
		align-content: start;

		padding: 10px;
		gap: 10px;
	}

	.clip {
		width: 150px;
		height: 75px;

		background-color: var(--secondary);

		border-radius: 5px;

		display: flex;
		justify-content: center;
		align-items: center;
	}
</style>
