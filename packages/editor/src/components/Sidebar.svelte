<script lang="ts">
	import { assets, meta } from '../stores/projectStore'
	import { hold, held, heldX, heldY } from '../stores/heldStore'
	import { clipsAtFrame } from '../stores/timelineStore'

	const tabs = ['Assets', 'Export', 'Inspect', 'Debug']
	let activeTab = tabs[0]

	function holdAsset(event: MouseEvent, assetId: string) {
		event.preventDefault()

		hold({
			type: 'asset',
			content: assetId,
			origin: 'sidebar',
		})
	}

	async function exportAnimation() {
		for (let frame = 0; frame < 900; frame++) {
			console.log(frame)

			const render = new OffscreenCanvas(1920, 1080)

			for (const clip of clipsAtFrame(frame)) {
				clip.asset.toFrame(frame - clip.frame + clip.firstClipFrame)

				await clip.asset.render(render)
			}

			const canvas = document.createElement('canvas')
			canvas.width = render.width
			canvas.height = render.height

			canvas.getContext('2d').drawImage(render, 0, 0)

			import.meta.hot.send('@vector-engine/export', { frame, dataUrl: canvas.toDataURL() })
		}
	}
</script>

<main>
	<div class="tabs">
		{#each tabs as tab}
			<button on:click={() => (activeTab = tab)} class={activeTab === tab ? 'active' : ''}
				>{tab}</button
			>
		{/each}
	</div>

	{#if activeTab === 'Assets'}
		<div class="content">
			{#each Object.keys($assets) as assetId}
				<div on:mousedown={event => holdAsset(event, assetId)} class="asset">
					<p>{$meta.assets[assetId].name}</p>
				</div>
			{/each}
		</div>

		{#if $held !== null && $held.origin === 'sidebar'}
			<div id="held-asset" class="asset" style="left: {$heldX}px; top: {$heldY}px;">
				<p>{$meta.assets[$held.content].name}</p>
			</div>
		{/if}
	{/if}

	{#if activeTab === 'Export'}
		<button on:click={exportAnimation}>Export</button>
	{/if}
</main>

<style>
	main {
		background-color: var(--main);

		grid-column-start: 1;
		grid-column-end: 2;
		grid-row-start: 1;
		grid-row-end: 2;

		border-right: solid 1px var(--secondary);
	}

	.tabs {
		display: flex;
		justify-content: space-between;

		border-bottom: solid 1px var(--secondary);
	}

	.tabs > button {
		flex-grow: 1;

		appearance: none;

		border: none;

		background: var(--main);

		height: 30px;

		cursor: pointer;
	}

	.tabs > button:hover {
		background: var(--secondary);
	}

	.tabs > button.active {
		background: var(--secondary);
	}

	.content {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-around;
		align-items: start;
		align-content: start;

		padding: 10px;
		gap: 10px;
	}

	.asset {
		width: 150px;
		height: 75px;

		background: var(--secondary);

		border-radius: 5px;

		display: flex;
		justify-content: center;
		align-items: center;

		cursor: pointer;
	}

	.asset > p {
		user-select: none;
	}

	#held-asset {
		position: absolute;

		transform: translate(-50%, -50%);

		filter: drop-shadow(0px 0px 8px #0000004a);
	}
</style>
