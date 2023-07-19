<script lang="ts">
	import { assets } from '../stores/projectStore'
	import { dropped, heldOn } from '../stores/heldStore'
	import { frame as globalFrame } from '../stores/playStateStore'
	import { clipsAtFrame, layers } from '../stores/timelineStore'
	import { Asset } from '@vector-engine/core'

	let previewingAssetId: string | null = null

	let canvas: HTMLCanvasElement

	let requestingRender = false
	let rendering = false

	async function render() {
		if (rendering) {
			requestingRender = true

			return
		}

		rendering = true

		const offscreenCanvas = new OffscreenCanvas(1920, 1080)

		if (previewingAssetId === null) {
			for (const clip of clipsAtFrame($globalFrame)) {
				clip.asset.toFrame($globalFrame - clip.frame)

				await clip.asset.render(offscreenCanvas)
			}
		} else {
			const asset: Asset = $assets[previewingAssetId]()
			asset.toFrame(0)

			await asset.render(offscreenCanvas)
		}

		const context = canvas.getContext('2d')

		context.clearRect(0, 0, 1920, 1080)
		context.drawImage(offscreenCanvas, 0, 0)

		rendering = false

		if (requestingRender) {
			requestingRender = false

			render()
		}
	}

	let componentBody: HTMLElement

	$: if ($dropped !== null && heldOn(componentBody) && $dropped.type === 'asset') {
		previewingAssetId = $dropped.content

		render()
	}

	$: if (canvas !== undefined && $assets !== undefined && $layers !== undefined) {
		if (previewingAssetId === null) render()
	}

	let lastRenderedGlobalFrame = -1

	$: if ($globalFrame !== lastRenderedGlobalFrame && canvas !== undefined) {
		lastRenderedGlobalFrame = $globalFrame

		previewingAssetId = null

		render()
	}
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
