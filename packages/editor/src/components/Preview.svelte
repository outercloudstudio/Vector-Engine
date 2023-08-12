<script lang="ts">
	import { assets } from '../stores/projectStore'
	import { dropped, heldOn } from '../stores/heldStore'
	import { frame as globalFrame } from '../stores/playStateStore'
	import { clipsAtFrame, timeline } from '../stores/timelineStore'
	import { Asset } from '@vector-engine/core'

	let previewingAssetId: string | null = null

	let canvas: HTMLCanvasElement

	let requestingRender = false
	let rendering = false

	async function update() {
		await previewAudio()
		await render()
	}

	async function previewAudio() {
		if (previewingAssetId === null) {
			for (const clip of clipsAtFrame($globalFrame)) {
				await clip.asset.previewAudio($globalFrame - clip.frame + clip.firstClipFrame)
			}
		} else {
			const asset: Asset = $assets[previewingAssetId]()
			asset.toFrame(0)

			await asset.previewAudio($globalFrame)
		}
	}

	async function render() {
		if (rendering) {
			requestingRender = true

			return
		}

		rendering = true

		const offscreenCanvas = new OffscreenCanvas(1920, 1080)

		if (previewingAssetId === null) {
			for (const clip of clipsAtFrame($globalFrame)) {
				clip.asset.toFrame($globalFrame - clip.frame + clip.firstClipFrame)

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

	$: if (
		canvas !== undefined &&
		$dropped !== null &&
		heldOn(componentBody) &&
		$dropped.type === 'asset'
	) {
		previewingAssetId = $dropped.content

		update()
	}

	$: if (canvas !== undefined && $assets !== undefined && $timeline !== undefined) {
		if (previewingAssetId === null) update()
	}

	let lastUpdatedGlobalFrame = -1

	$: if (canvas !== undefined && $globalFrame !== lastUpdatedGlobalFrame && canvas !== undefined) {
		lastUpdatedGlobalFrame = $globalFrame

		previewingAssetId = null

		update()
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
