<script lang="ts">
	import { subscribe, tick } from 'svelte/internal'
	import { engine, engineData, frame } from '../Stores/EngineStore'
	import { get } from 'svelte/store'
	import { pause } from '../Stores/PlayStore'
	import { audioInference, markerInference, sceneInference } from '../Stores/InferenceStore'

	let canvas: undefined | HTMLCanvasElement = undefined
	let containerWidth = 1920
	let containerHeight = 1080

	let offset = 0
	let scale = 1

	let mouse = false

	async function mouseDown(event: MouseEvent) {
		mouse = true

		const currentEngine = get(engine)

		const newFrame = Math.max(
			0,
			Math.min(currentEngine.length - 1, Math.round(getFrameAtXPosition(event.clientX)))
		)

		pause()

		await currentEngine.jumpToFrame(newFrame)
		frame.set(newFrame)
	}

	function mouseMove(event: MouseEvent) {
		if (!mouse) return

		pause()

		const newFrame = Math.max(
			0,
			Math.min(get(engine).length - 1, Math.round(getFrameAtXPosition(event.clientX)))
		)

		frame.set(newFrame)
	}

	async function mouseUp(event: MouseEvent) {
		mouse = false

		const currentFrame = get(frame)
		const currentEngine = get(engine)

		if (currentEngine === undefined) return

		await currentEngine.jumpToFrame(currentFrame)
		frame.set(-1)
		frame.set(currentFrame)
	}

	function getFrameAtXPosition(x: number): number {
		return (x - offset) / 5 / scale
	}

	function getXPositionOfFrame(frame: number): number {
		return frame * 5 * scale + offset
	}

	function scroll(event: any) {
		const canvasBounds = canvas.getBoundingClientRect()
		const mouseOffset = event.clientX - canvasBounds.left - getXPositionOfFrame(0)
		const scrollX = event.deltaX / -36
		const scrollY = event.deltaY / -4000

		const zoom = Math.exp(scrollY)

		offset += mouseOffset * (1 - zoom)

		scale *= zoom

		offset += (canvas.width / 25) * scrollX
	}

	const mainColor = '#141414'
	const secondaryColor = '#242424'
	const tertiaryColor = '#1c1c1c'
	const grabColor = '#32a6fc'
	const alternateGrabColor = '#17222b'
	const textColor = '#d9d9d9'
	const alternateTextColor = '#a7a7a7'

	function renderFrameLabels(canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D) {
		const frameWidth =
			canvas && scale ? getFrameAtXPosition(canvas.width) - getFrameAtXPosition(0) : 0

		const interval = Math.max(
			Math.floor(Math.pow(2, Math.floor(Math.log(frameWidth) / Math.log(2))) / 8),
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
			ctx.font = `${10}px Mulish`

			const measure = ctx.measureText(frame.toString())

			ctx.fillText(
				frame.toString(),
				x - measure.actualBoundingBoxRight / 2,
				10 + measure.actualBoundingBoxAscent / 2
			)
		}
	}

	function renderFrameLines(canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D) {
		const frameWidth =
			canvas && scale ? getFrameAtXPosition(canvas.width) - getFrameAtXPosition(0) : 0

		const interval = Math.max(
			Math.floor(Math.pow(2, Math.floor(Math.log(frameWidth) / Math.log(2))) / 8),
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

	function renderAudio(canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D) {
		const currentAudioInference = get(audioInference)

		for (
			let frame = Math.floor(getFrameAtXPosition(0));
			frame < Math.ceil(getFrameAtXPosition(canvas.width));
			frame++
		) {
			if (frame < 0 || frame >= get(engine).length) continue

			const x = getXPositionOfFrame(frame)

			ctx.fillStyle = secondaryColor
			ctx.fillRect(
				x,
				canvas.height,
				getXPositionOfFrame(frame + 1) - getXPositionOfFrame(frame),
				-100 * currentAudioInference[frame]
			)
		}
	}

	function renderPlayhead(canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D) {
		ctx.fillStyle = grabColor
		ctx.fillRect(getXPositionOfFrame(get(frame)), 0, 1, canvas.height)
	}

	function renderValidArea(canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D) {
		ctx.fillStyle = tertiaryColor
		ctx.fillRect(
			getXPositionOfFrame(0),
			0,
			getXPositionOfFrame(get(engine).length - 1) - getXPositionOfFrame(0),
			canvas.height
		)
	}

	async function render() {
		if (!canvas) return
		if (get(engine) === undefined) return

		const ctx = canvas.getContext('2d')!
		ctx.imageSmoothingEnabled = false

		ctx.clearRect(0, 0, canvas.width, canvas.height)

		renderValidArea(canvas, ctx)
		renderFrameLabels(canvas, ctx)
		renderFrameLines(canvas, ctx)
		renderAudio(canvas, ctx)
		renderPlayhead(canvas, ctx)
	}

	subscribe(engine, value => {
		if (value === undefined) return

		scale = 1
		offset = 0

		const frameWidth =
			canvas && scale ? getFrameAtXPosition(canvas.width) - getFrameAtXPosition(0) : 0

		scale = frameWidth / (value.length - 1) / 1.5
		offset = canvas.width / 1.5 / 4
	})

	subscribe(frame, () => {
		render()
	})

	subscribe(audioInference, () => {
		render()
	})

	$: containerWidth, containerHeight, tick().then(render)
	$: scale, offset, render()
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

	{#if $engineData}
		{#each $sceneInference as scene}
			<p
				class="scene"
				style="left: {scene.frame * 5 * scale + offset}px; width:calc({scene.length *
					5 *
					scale}px - 0.4rem)"
			>
				{scene.name}
			</p>
		{/each}

		{#each $markerInference as marker}
			<div
				class="marker-length"
				style="left: {marker.frame * 5 * scale + offset}px; width:calc({marker.length *
					5 *
					scale}px + 0.4rem)"
			/>
			<p class="marker" style="left: {(marker.frame + marker.length) * 5 * scale + offset}px">
				{marker.name}
			</p>
		{/each}
	{/if}
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

		position: relative;
	}

	canvas {
		background: var(--main);
		image-rendering: pixelated;

		display: block;

		width: 100%;
		height: 100%;
	}

	.marker-length {
		background-color: var(--secondary);

		border-radius: 0.2rem;

		position: absolute;
		top: 52px;

		height: 1.3rem;
	}

	.marker {
		background-color: var(--grab);

		border-radius: 0.4rem;
		border-top-left-radius: 0;

		margin: 0;

		padding: 0.2rem;
		font-size: 0.7rem;

		display: table;

		position: absolute;
		top: 52px;

		z-index: 1;
	}

	.scene {
		background-color: var(--secondary);

		border-radius: 0.2rem;

		margin: 0;

		padding: 0.2rem;
		font-size: 0.7rem;

		position: absolute;
		top: 24px;

		color: var(--alternate-text);
	}
</style>
