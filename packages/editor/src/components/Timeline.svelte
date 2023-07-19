<script lang="ts">
	import { heldOn, heldX, heldY, held, dropped, hold, type Holdable } from '../stores/heldStore'
	import { frame as globalFrame, play, playing, pause } from '../stores/playStateStore'
	import { assets } from '../stores/projectStore'
	import {
		layers,
		addClip,
		removeClip,
		findClipLocation,
		nextValidFrame,
	} from '../stores/timelineStore'

	let componentMainElement: HTMLElement
	let clipsElement: HTMLElement

	let viewStartFrame = -15
	let viewFrameLength = 360
	let layerOffset = 0

	$: frameToPixelOffset = function (frame: number) {
		if (componentMainElement === undefined) return 0

		return (
			(componentMainElement.getBoundingClientRect().width * (frame - viewStartFrame)) / viewFrameLength
		)
	}

	$: framesToPixels = function (frame: number) {
		if (componentMainElement === undefined) return 0

		return (componentMainElement.getBoundingClientRect().width * frame) / viewFrameLength
	}

	$: pixelOffsetToFrame = function (x: number) {
		if (componentMainElement === undefined) return 0

		return Math.floor(
			(x / componentMainElement.getBoundingClientRect().width) * viewFrameLength + viewStartFrame
		)
	}

	$: pixelOffsetToFrameRounded = function (x: number) {
		if (componentMainElement === undefined) return 0

		return Math.round(
			(x / componentMainElement.getBoundingClientRect().width) * viewFrameLength + viewStartFrame
		)
	}

	$: pixelOffsetToFrameContinuous = function (x: number) {
		if (componentMainElement === undefined) return 0

		return (x / componentMainElement.getBoundingClientRect().width) * viewFrameLength + viewStartFrame
	}

	$: layerToPixelOffset = function (layer: number) {
		if (componentMainElement === undefined) return 0

		return (
			componentMainElement.getBoundingClientRect().height / 2 + (layerOffset - layer) * 38 - 32 / 2
		)
	}

	$: pixelOffsetToLayer = function (y: number) {
		if (componentMainElement === undefined) return 0

		return Math.floor(
			(componentMainElement.getBoundingClientRect().height / 2 - y + 32 / 2) / 38 + layerOffset
		)
	}

	function globalYtoLocalY(y: number) {
		if (clipsElement === undefined) return 0

		return y - clipsElement.getBoundingClientRect().y
	}

	function localYToGlobalY(y: number) {
		if (clipsElement === undefined) return 0

		return clipsElement.getBoundingClientRect().y + y
	}

	function wheelScroll(event: WheelEvent) {
		if (!event.shiftKey) {
			layerOffset -= event.deltaY / 100 / 4
		} else {
			const mouseFrame = pixelOffsetToFrame(event.clientX)
			const mouseXFactor = (mouseFrame - viewStartFrame) / viewFrameLength

			viewFrameLength = Math.exp(Math.log(viewFrameLength) + event.deltaY / 5000)

			const newFrameAtFactor = viewStartFrame + viewFrameLength * mouseXFactor

			viewStartFrame += mouseFrame - newFrameAtFactor
		}

		viewStartFrame += ((event.deltaX / 36) * viewFrameLength) / 20 / 4
	}

	let timeLines = []

	// Renders timeline elements
	$: if (componentMainElement !== undefined) {
		let newTimeLines = []

		const frameWidth = pixelOffsetToFrameContinuous(128) - pixelOffsetToFrameContinuous(0)

		const frameStep = frameWidth >= 1 ? Math.ceil(frameWidth / 5) * 5 : 1

		const firstLabel = Math.floor(pixelOffsetToFrame(0) / frameStep) * frameStep

		for (
			let frame = firstLabel;
			frame <= Math.ceil(viewStartFrame) + viewFrameLength + frameStep;
			frame += frameStep
		) {
			newTimeLines.push(frame)
		}

		timeLines = newTimeLines
	}

	let originalHeldObject: Holdable | null = null

	// Handles a held asset being held over the timeline
	$: if (
		heldOn(componentMainElement) &&
		$heldX !== undefined &&
		$heldY !== undefined &&
		$held &&
		$held.type === 'asset' &&
		$held.origin !== 'timeline'
	) {
		originalHeldObject = $held

		held.set({
			type: 'asset',
			content: originalHeldObject.content,
			origin: 'timeline',
		})

		heldClipOffset = 0
	}

	// Handles a held asset no longer being held over the timeline
	$: if (
		!heldOn(componentMainElement) &&
		$heldX !== undefined &&
		$heldY !== undefined &&
		$held &&
		$held.origin === 'timeline' &&
		originalHeldObject !== null
	) {
		held.set(originalHeldObject)

		originalHeldObject = null
	}

	let heldClipOffset = 0

	// Handles held objects being dropped onto the timeline
	$: if ($dropped !== null && heldOn(componentMainElement)) {
		if ($dropped.type === 'asset') {
			addClip(
				$dropped.content,
				$assets[$dropped.content](),
				pixelOffsetToFrame($heldX),
				60,
				pixelOffsetToLayer(globalYtoLocalY($heldY))
			)

			originalHeldObject = null
		} else {
			addClip(
				$dropped.content.assetId,
				$assets[$dropped.content.assetId](),
				pixelOffsetToFrame($heldX + framesToPixels(heldClipOffset)),
				$dropped.content.length,
				pixelOffsetToLayer(globalYtoLocalY($heldY)),
				$dropped.content.id
			)
		}

		dropped.set(null)
	}

	function holdClip(event: MouseEvent, clip: any) {
		event.preventDefault()

		hold({
			type: 'clip',
			content: clip,
			origin: 'timeline',
		})

		removeClip(clip.id)

		heldClipOffset = clip.frame - pixelOffsetToFrameContinuous($heldX)
	}

	let resizeClipId: string | null = null
	let resizeClipSide: 'left' | 'right' = 'right'

	function resizeClipLeft(id: string) {
		resizeClipSide = 'left'
		resizeClipId = id
	}

	function resizeClipRight(id: string) {
		resizeClipSide = 'right'
		resizeClipId = id
	}

	function resizeClipMouseUp() {
		resizeClipId = null
	}

	function resizeClipMouseMove(event: MouseEvent) {
		if (resizeClipId === null) return

		const frame = pixelOffsetToFrameRounded(event.clientX)

		if (resizeClipSide === 'right') {
			const clipLocation = findClipLocation(resizeClipId)

			let newLength = frame - $layers[clipLocation.layer][clipLocation.index].frame

			if (newLength < 1) newLength = 1

			if (
				$layers[clipLocation.layer][clipLocation.index + 1] !== undefined &&
				frame > $layers[clipLocation.layer][clipLocation.index + 1].frame
			)
				newLength =
					$layers[clipLocation.layer][clipLocation.index + 1].frame -
					$layers[clipLocation.layer][clipLocation.index].frame

			$layers[clipLocation.layer][clipLocation.index].length = newLength
		}

		if (resizeClipSide === 'left') {
			const clipLocation = findClipLocation(resizeClipId)

			const oldStart = $layers[clipLocation.layer][clipLocation.index].frame

			let newStart = frame
			if (newStart >= oldStart + $layers[clipLocation.layer][clipLocation.index].length)
				newStart = oldStart + $layers[clipLocation.layer][clipLocation.index].length - 1

			if (newStart < 0) newStart = 0

			if (
				$layers[clipLocation.layer][clipLocation.index - 1] !== undefined &&
				frame <
					$layers[clipLocation.layer][clipLocation.index - 1].frame +
						$layers[clipLocation.layer][clipLocation.index - 1].length
			)
				newStart =
					$layers[clipLocation.layer][clipLocation.index - 1].frame +
					$layers[clipLocation.layer][clipLocation.index - 1].length

			$layers[clipLocation.layer][clipLocation.index].frame = newStart
			$layers[clipLocation.layer][clipLocation.index].length =
				$layers[clipLocation.layer][clipLocation.index].length + (oldStart - newStart)
		}
	}

	let movePlayheadMouseDown = false

	function movePlayheadOnMouseDown(event: MouseEvent) {
		if ($held !== null) return

		movePlayheadMouseDown = true

		pause()
		globalFrame.set(Math.max(0, pixelOffsetToFrameRounded(event.clientX)))
	}

	function movePlayheadOnMouseUp(event: MouseEvent) {
		if ($held !== null) return

		if (resizeClipId !== null) return

		movePlayheadMouseDown = false

		globalFrame.set(Math.max(0, pixelOffsetToFrameRounded(event.clientX)))
	}

	function movePlayheadOnMouseMove(event: MouseEvent) {
		if ($held !== null) return

		if (!movePlayheadMouseDown) return

		globalFrame.set(Math.max(0, pixelOffsetToFrameRounded(event.clientX)))
	}
</script>

<main bind:this={componentMainElement}>
	<div class="control-bar">
		<button
			on:click={() => globalFrame.set(Math.max(0, $globalFrame - 1))}
			class="material-symbols-outlined"
		>
			skip_previous
		</button>

		{#if !$playing}
			<button on:click={play} class="material-symbols-outlined"> play_arrow </button>
		{:else}
			<button on:click={pause} class="material-symbols-outlined"> pause </button>
		{/if}

		<button on:click={() => globalFrame.set($globalFrame + 1)} class="material-symbols-outlined">
			skip_next
		</button>
	</div>

	<div
		on:wheel={wheelScroll}
		on:mousedown={movePlayheadOnMouseDown}
		on:mouseup={movePlayheadOnMouseUp}
		on:mouseup={resizeClipMouseUp}
		on:mousemove={movePlayheadOnMouseMove}
		on:mousemove={resizeClipMouseMove}
		class="timeline"
	>
		{#each timeLines as line}
			<div
				class="time-line"
				style="left: {frameToPixelOffset(line)}px; z-index: {line === 0 ? 2 : 0};"
			>
				<div class="time-line-text">
					<p class="time">{Math.floor((line / 60) * 100) / 100}</p>
					<p class="frame">[{line}]</p>
				</div>
				<div class="line" />
			</div>
		{/each}

		<div bind:this={clipsElement} class="clips">
			{#each Object.keys($layers).map(layer => parseInt(layer)) as layer}
				{#each $layers[layer] as clip}
					<div
						on:mousedown={event => holdClip(event, clip)}
						class="clip"
						style="left: {frameToPixelOffset(clip.frame)}px; width: {framesToPixels(
							clip.length
						)}px; top: {layerToPixelOffset(layer)}px"
					>
						<p>{clip.assetId}</p>

						<div on:mousedown|stopPropagation={() => resizeClipLeft(clip.id)} class="resize-left" />
						<div on:mousedown|stopPropagation={() => resizeClipRight(clip.id)} class="resize-right" />
					</div>
				{/each}
			{/each}
		</div>

		<div class="play-head" style="left: {frameToPixelOffset($globalFrame)}px;" />

		<div class="zero-cover" style="left: {frameToPixelOffset(0)}px" />
	</div>

	{#if $held !== null && $held.origin === 'timeline' && heldOn(clipsElement) && $heldX !== undefined && $heldY !== undefined}
		<div
			class="clip"
			style="left: {frameToPixelOffset(
				nextValidFrame(
					pixelOffsetToFrame($heldX + framesToPixels(heldClipOffset)),
					60,
					pixelOffsetToLayer(globalYtoLocalY($heldY))
				)
			)}px; width: {framesToPixels(
				$held.type === 'asset' ? 60 : $held.content.length
			)}px; top: {localYToGlobalY(layerToPixelOffset(pixelOffsetToLayer(globalYtoLocalY($heldY))))}px"
		>
			<p>{$held.type === 'asset' ? $held.content : $held.content.assetId}</p>
		</div>
	{/if}

	{#if $held !== null && $held.origin === 'timeline' && !heldOn(clipsElement) && $heldX !== undefined && $heldY !== undefined}
		<div
			class="clip"
			style="left: {$heldX + framesToPixels(heldClipOffset)}px; width: {framesToPixels(
				60
			)}px; top: {$heldY - 16}px"
		>
			<p>{$held.type === 'asset' ? $held.content : $held.content.assetId}</p>
		</div>
	{/if}
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

		z-index: 3;

		cursor: move;
	}

	.clip > p {
		text-align: center;

		user-select: none;
	}

	.resize-left {
		width: 8px;
		height: 100%;

		cursor: col-resize;

		position: absolute;

		left: 0;
	}

	.resize-right {
		width: 8px;
		height: 100%;

		cursor: col-resize;

		position: absolute;

		right: 0;
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

		user-select: none;
	}

	.time-line-text > .frame {
		display: table;

		font-size: small;

		margin: 0;

		position: absolute;

		left: calc(100% + 2px);
		bottom: 2px;

		color: var(--secondary-text);

		user-select: none;
	}

	.line {
		width: 1px;

		flex-grow: 1;

		background: var(--secondary);
	}

	.zero-cover {
		z-index: 1;

		position: absolute;

		background: #00000033;

		width: 100%;
		height: 100%;

		top: 0;

		translate: -100% 0%;
	}

	.play-head {
		z-index: 4;

		width: 1px;

		background: #ffffff;

		height: 100%;

		position: absolute;

		top: 0;

		pointer-events: none;
	}
</style>
