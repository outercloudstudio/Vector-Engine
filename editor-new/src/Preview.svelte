<script lang="ts">
	import { subscribe } from 'svelte/internal'
	import { engine, frame } from './stores.js'
	import { get } from 'svelte/store'
	import { getBestFittingBounds } from './aspectRatio.js'

	let canvas: undefined | HTMLCanvasElement = undefined
	let containerWidth = 1920
	let containerHeight = 1080

	let offsetX = 0
	let offsetY = 0
	let scale = 1

	let mouse = false
	let mouseDownX = 0
	let mouseDownY = 0

	function mouseDown(event: MouseEvent) {
		mouse = true

		mouseDownX = event.clientX
		mouseDownY = event.clientY
	}

	function mouseMove(event: MouseEvent) {
		if (!mouse) return

		offsetX += event.clientX - mouseDownX
		offsetY += event.clientY - mouseDownY

		mouseDownX = event.clientX
		mouseDownY = event.clientY
	}

	function mouseUp(event: MouseEvent) {
		mouse = false
	}

	function scroll(event: any) {
		const canvasBounds = canvas.getBoundingClientRect()
		const mouseOffsetX = event.clientX - (canvasBounds.left + canvasBounds.right) / 2
		const mouseOffsetY = event.clientY - (canvasBounds.top + canvasBounds.bottom) / 2
		const scrollY = event.deltaY / -4000

		const zoom = Math.exp(scrollY)

		const width = canvasBounds.right - canvasBounds.left
		const widthDelta = width * zoom - width
		const offsetXFactor = (mouseOffsetX / width) * 2
		offsetX -= (widthDelta / 2) * offsetXFactor

		const height = canvasBounds.top - canvasBounds.bottom
		const heightDelta = height * zoom - height
		const offsetYFactor = (mouseOffsetY / height) * 2
		offsetY -= (heightDelta / 2) * offsetYFactor

		scale *= zoom
	}

	async function render() {
		if (!canvas) return

		const engineValue = get(engine)

		if (engineValue === undefined) return

		const render = await engineValue.render()

		const ctx = canvas.getContext('2d')!
		ctx.imageSmoothingEnabled = false

		ctx.clearRect(0, 0, canvas.width, canvas.height)

		ctx.drawImage(render, 0, 0, canvas.width, canvas.height)
	}

	subscribe(engine, () => {
		render()
	})

	subscribe(frame, () => {
		render()
	})

	$: if (canvas) {
		const size = getBestFittingBounds(containerWidth, containerHeight, 16 / 9)

		canvas.style.maxWidth = size.width + 'px'
		canvas.style.maxHeight = size.height + 'px'

		render()
	}

	$: if (canvas) {
		canvas.style.translate = offsetX + 'px' + ' ' + offsetY + 'px'
		canvas.style.scale = scale.toString()
	}
</script>

<div
	bind:clientWidth={containerWidth}
	bind:clientHeight={containerHeight}
	on:mousedown={mouseDown}
	on:mousemove={mouseMove}
	on:mouseleave={mouseUp}
	on:mouseup={mouseUp}
	on:wheel={scroll}
>
	<canvas bind:this={canvas} width={1920} height={1080} />
</div>

<style>
	div {
		background: #000000;
		height: 50%;

		display: flex;
		justify-content: center;
		align-items: center;

		overflow: hidden;
	}

	canvas {
		background-image: url('./Assets/checker.png');
		background-repeat: repeat;
		background-position: bottom left;
		image-rendering: pixelated;

		display: block;

		width: 100%;
		height: 100%;
	}
</style>
