<script lang="ts">
	import { onMount } from 'svelte'
	import { scenes } from '../stores/projectStore'

	let canvas: HTMLCanvasElement

	scenes.subscribe(scenes => {
		if (scenes.scene === undefined) return

		const offscreenCanvas = new OffscreenCanvas(1920, 1080)

		scenes.scene.render(offscreenCanvas)

		const context = canvas.getContext('2d')

		context.clearRect(0, 0, 1920, 1080)
		context.drawImage(offscreenCanvas, 0, 0)
	})
</script>

<main>
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
