<script lang="ts">
	import { assets } from '../stores/projectStore'
	import { dropped, heldOn } from '../stores/heldStore'

	let previewingAssetId: string | null = null

	let canvas: HTMLCanvasElement

	function render() {
		if (previewingAssetId === null) return

		const offscreenCanvas = new OffscreenCanvas(1920, 1080)

		$assets[previewingAssetId].render(offscreenCanvas)

		const context = canvas.getContext('2d')

		context.clearRect(0, 0, 1920, 1080)
		context.drawImage(offscreenCanvas, 0, 0)
	}

	let componentBody: HTMLElement

	$: if ($dropped !== null && heldOn(componentBody) && $dropped.type === 'asset') {
		previewingAssetId = $dropped.content

		render()
	}

	assets.subscribe(assets => {
		if (previewingAssetId === null && Object.keys(assets).length > 0)
			previewingAssetId = Object.keys(assets)[0]

		render()
	})
</script>

<main bind:this={componentBody}>
	<canvas bind:this={canvas} width="1920" height="1080" />
</main>

<style>
	main {
		background-color: var(--main);

		grid-column-start: 2;
		grid-column-end: 3;
		grid-row-start: 1;
		grid-row-end: 2;
	}

	canvas {
		display: block;
		max-width: 100%;
		max-height: 100%;
		width: auto;
		height: auto;
		margin: auto;

		background: white;
	}
</style>
