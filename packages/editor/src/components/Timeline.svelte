<script lang="ts">
	import { assets } from '../stores/projectStore'

	let clips = []

	$: if (Object.keys($assets).length > 0) {
		clips = [
			{
				assetId: Object.keys($assets)[0],
				start: 5,
				end: 20,
				layer: 0,
			},
			{
				assetId: Object.keys($assets)[0],
				start: 15,
				end: 30,
				layer: 1,
			},
		]
	}
</script>

<main>
	<div class="control-bar">
		<button class="material-symbols-outlined"> skip_previous </button>
		<button class="material-symbols-outlined"> play_arrow </button>
		<button class="material-symbols-outlined"> skip_next </button>
	</div>

	<div class="clips">
		{#each clips as clip}
			<div
				class="clip"
				style="left: {clip.start * 40}px; width: {(clip.end - clip.start) * 40}px; bottom: {clip.layer *
					38}px"
			>
				<p>{clip.assetId}</p>
			</div>
		{/each}
	</div>
</main>

<style>
	main {
		background-color: var(--main);

		grid-column-start: 1;
		grid-column-end: 3;
		grid-row-start: 2;
		grid-row-end: 3;

		border-top: solid 1px var(--secondary);

		display: flex;
		flex-direction: column;
	}

	.control-bar {
		display: flex;
		justify-content: center;
		align-items: center;

		padding: 4px;

		border-bottom: solid 1px var(--secondary);
	}

	.control-bar > button {
		background: none;
		border: none;

		cursor: pointer;
	}

	.clips {
		position: relative;

		flex-grow: 1;

		margin: 8px;
	}

	.clip {
		height: 32px;

		border-radius: 5px;

		background: var(--secondary);

		overflow: hidden;

		display: flex;
		justify-content: center;
		align-items: center;

		position: absolute;
	}

	.clip > p {
		text-align: center;
	}
</style>
