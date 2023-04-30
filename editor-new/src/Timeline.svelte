<script lang="ts">
	import { subscribe } from 'svelte/internal'
	import { frame } from './stores'
	import { getBestFittingBounds } from './aspectRatio'
	import { get } from 'svelte/store'

	let canvas: undefined | HTMLCanvasElement = undefined
	let containerWidth = 1920
	let containerHeight = 1080

	let offset = 0
	let scale = 1
	$: frameWidth = canvas && scale ? getFrameAtXPosition(canvas.width) - getFrameAtXPosition(0) : 0

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

		offset += event.clientX - mouseDownX

		mouseDownX = event.clientX
		mouseDownY = event.clientY
	}

	function mouseUp(event: MouseEvent) {
		mouse = false
	}

	function scroll(event: any) {
		const canvasBounds = canvas.getBoundingClientRect()
		const mouseOffsetX = event.clientX - (canvasBounds.left + canvasBounds.right) / 2
		const scrollY = event.deltaY / -4000

		const zoom = Math.exp(scrollY)

		// const width = canvasBounds.right - canvasBounds.left
		// const widthDelta = width * zoom - width
		// const offsetXFactor = (mouseOffsetX / width) * 2
		// offsetX -= (widthDelta / 2) * offsetXFactor

		// const height = canvasBounds.top - canvasBounds.bottom
		// const heightDelta = height * zoom - height
		// const offsetYFactor = (mouseOffsetY / height) * 2
		// offsetY -= (heightDelta / 2) * offsetYFactor

		scale *= zoom
	}

	const mainColor = '#141414'
	const secondaryColor = '#242424'
	const grabColor = '#32a6fc'
	const alternateGrabColor = '#17222b'
	const textColor = '#d9d9d9'
	const alternateTextColor = '#a7a7a7'

	function getFrameAtXPosition(x: number): number {
		return (x - offset) / 5 / scale
	}

	function getXPositionOfFrame(frame: number): number {
		return frame * 5 * scale + offset
	}

	function renderFrameLabels(canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D) {
		const interval = Math.max(
			Math.floor(Math.pow(2, Math.floor(Math.log(frameWidth) / Math.log(2))) / (canvas.width / 80)),
			1
		)
		const firstLabel = Math.floor(getFrameAtXPosition(0) / interval) * interval

		for (
			let frame = firstLabel;
			frame < Math.ceil(getFrameAtXPosition(canvas.width));
			frame += interval
		) {
			const x = getXPositionOfFrame(frame)

			ctx.fillStyle = alternateTextColor
			ctx.font = `${10}px system-ui`

			const measure = ctx.measureText(frame.toString())

			ctx.fillText(
				frame.toString(),
				x - measure.actualBoundingBoxRight / 2,
				10 + measure.actualBoundingBoxAscent / 2
			)
		}
	}

	function renderFrameLines(canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D) {
		const interval = Math.max(
			Math.floor(Math.pow(2, Math.floor(Math.log(frameWidth) / Math.log(2))) / (canvas.width / 80)),
			1
		)
		const firstLabel = Math.floor(getFrameAtXPosition(0) / interval) * interval

		for (
			let frame = firstLabel;
			frame < Math.ceil(getFrameAtXPosition(canvas.width));
			frame += interval
		) {
			const x = getXPositionOfFrame(frame)

			ctx.fillStyle = secondaryColor
			ctx.fillRect(x, 20, 1, canvas.height - 20)
		}
	}

	function renderPlayhead(canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D) {
		const x = getXPositionOfFrame(get(frame))

		ctx.fillStyle = grabColor
		ctx.fillRect(x, 0, 1, canvas.height)
	}

	async function render() {
		if (!canvas) return

		const ctx = canvas.getContext('2d')!
		ctx.imageSmoothingEnabled = false

		ctx.clearRect(0, 0, canvas.width, canvas.height)

		renderFrameLabels(canvas, ctx)
		renderFrameLines(canvas, ctx)
		renderPlayhead(canvas, ctx)
	}

	subscribe(frame, () => {
		render()
	})

	$: if (canvas && containerWidth && containerHeight) {
		render()
	}
</script>

<main
	bind:clientWidth={containerWidth}
	bind:clientHeight={containerHeight}
	on:mousedown={mouseDown}
	on:mousemove={mouseMove}
	on:mouseleave={mouseUp}
	on:mouseup={mouseUp}
	on:wheel={scroll}
>
	<canvas bind:this={canvas} width={containerWidth} height={containerHeight} />
</main>

<style>
	main {
		background: #000000;
		max-height: 50%;
		height: 50%;

		display: flex;
		justify-content: center;
		align-items: center;

		overflow: hidden;
	}

	canvas {
		background: var(--main);
		image-rendering: pixelated;

		display: block;

		width: 100%;
		height: 100%;
	}
</style>
