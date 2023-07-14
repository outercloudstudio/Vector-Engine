<script lang="ts">
	import { assets } from '../stores/projectStore'

	let componentMainElement: HTMLElement

	let viewStartFrame = 0
	let viewFrameLength = 60
	let layerOffset = 0

	let clips = []
	let timeLines = []

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

		timeLines = [0, 20, 40, 60]
	}

	$: frameToPixelOffset = function (frame: number) {
		if (componentMainElement === undefined) return 0

		return (
			(componentMainElement.getBoundingClientRect().width * (frame - viewStartFrame)) / viewFrameLength
		)
	}

	$: pixelOffsetToFrame = function (x: number) {
		if (componentMainElement === undefined) return 0

		return (x / componentMainElement.getBoundingClientRect().width) * viewFrameLength + viewStartFrame
	}

	$: layerToPixelOffset = function (layer: number) {
		if (componentMainElement === undefined) return 0

		return (
			componentMainElement.getBoundingClientRect().height / 2 + (layer - layerOffset) * 38 - 32 / 2
		)
	}

	function wheelScroll(event: WheelEvent) {
		if (!event.shiftKey) {
			layerOffset -= event.deltaY / 100 / 4
		} else {
			const mouseFrame = pixelOffsetToFrame(event.clientX)
			const mouseXFactor = (mouseFrame - viewStartFrame) / viewFrameLength

			viewFrameLength += event.deltaY / 100

			const newFrameAtFactor = viewStartFrame + viewFrameLength * mouseXFactor

			viewStartFrame += mouseFrame - newFrameAtFactor
		}

		viewStartFrame += ((event.deltaX / 36) * viewFrameLength) / 20
	}
</script>

<main bind:this={componentMainElement}>
	<div class="control-bar">
		<button class="material-symbols-outlined"> skip_previous </button>
		<button class="material-symbols-outlined"> play_arrow </button>
		<button class="material-symbols-outlined"> skip_next </button>
	</div>

	<div class="timeline" on:wheel={wheelScroll}>
		{#each timeLines as line}
			<div class="time-line" style="left: {frameToPixelOffset(line)}px;">
				<div class="time-line-text">
					<p class="time">{Math.floor((line / 60) * 100) / 100}</p>
					<p class="frame">[{line}]</p>
				</div>
				<div class="line" />
			</div>
		{/each}

		<div class="clips">
			{#each clips as clip}
				<div
					class="clip"
					style="left: {frameToPixelOffset(clip.start)}px; width: {frameToPixelOffset(clip.end) -
						frameToPixelOffset(clip.start)}px; bottom: {layerToPixelOffset(clip.layer)}px"
				>
					<p>{clip.assetId}</p>
				</div>
			{/each}
		</div>
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

	.timeline {
		position: relative;

		flex-grow: 1;
	}

	.clips {
		margin-top: 32px;

		overflow: hidden;

		height: 100%;

		position: relative;
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

	.time-line {
		display: table;

		position: absolute;

		top: 0px;

		translate: -50% 0;

		height: 100%;

		display: flex;
		flex-direction: column;
		align-items: center;
	}

	.time-line-text {
		display: flex;

		align-items: baseline;

		position: relative;

		margin-bottom: 4px;
	}

	.time-line-text > .time {
		display: table;

		margin: 0;
	}

	.time-line-text > .frame {
		display: table;

		font-size: small;

		margin: 0;

		position: absolute;

		left: calc(100% + 2px);
		bottom: 2px;

		color: var(--secondary-text);
	}

	.line {
		width: 1px;

		flex-grow: 1;

		background: var(--secondary);
	}
</style>
