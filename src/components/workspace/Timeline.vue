<template>
  <div id="component">
    <div id="control-bar">
      <div class="control-bar-group">
        <input value="0:00" id="time-input" />
        <input value="[0]" id="frame-input" />
      </div>

      <div class="control-bar-group">
        <span class="material-symbols-outlined icon-button"> volume_up </span>

        <span class="material-symbols-outlined icon-button"> fast_rewind </span>
      </div>

      <div class="control-bar-group">
        <span class="material-symbols-outlined icon-button">
          skip_previous
        </span>

        <span class="material-symbols-outlined icon-button"> play_arrow </span>

        <span class="material-symbols-outlined icon-button"> skip_next </span>
      </div>

      <div class="control-bar-group">
        <span class="material-symbols-outlined icon-button"> redo </span>
      </div>

      <div class="control-bar-group">
        <p id="frame-length">[60]</p>
        <p id="length">0:01</p>
      </div>
    </div>
    <div ref="canvasWrapper" id="canvas-wrapper">
      <canvas
        ref="canvas"
        id="canvas"
        @mousedown="mouseDown"
        @mouseup="() => (mouse = false)"
        @mouseleave="() => (mouse = false)"
        @mousemove="mouseMove"
        @wheel="scroll"
      ></canvas>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, Ref, ref, watch } from 'vue'

import { useWorkspaceStore } from '@/stores/WorkspaceStore'

const WorkspaceStore = useWorkspaceStore()

let mouse = false

function mouseDown(event: MouseEvent) {
  mouse = true

  mouseMove(event)
}

function mouseMove(event: MouseEvent) {
  if (!mouse) return
  if (!canvas.value) return

  const factor =
    (event.clientX - canvas.value.getBoundingClientRect().left) /
    canvas.value.width

  WorkspaceStore.frame = Math.floor(
    factor * (endFrame.value - startFrame.value) + startFrame.value
  )
}

let startFrame = ref(0)
let endFrame = ref(60)

function scroll(event: any) {
  const scrollX = Math.ceil(event.deltaX / 100)
  const scrollY = Math.ceil(event.deltaY / 100)

  let viewRange = endFrame.value - startFrame.value

  if (scrollY != 0) {
    let zoomAmount =
      (Math.ceil(
        Math.abs(Math.pow(Math.log(viewRange / 60), Math.abs(scrollY)))
      ) *
        scrollY) /
      Math.abs(scrollY)

    if (zoomAmount == 0) zoomAmount = 1 * (scrollY / Math.abs(scrollY))

    const midPoint = Math.floor(
      startFrame.value + (endFrame.value - startFrame.value) / 2
    )

    startFrame.value = startFrame.value - zoomAmount
    endFrame.value = endFrame.value + zoomAmount

    if (endFrame.value - startFrame.value <= 1) {
      startFrame.value = midPoint
      endFrame.value = midPoint + 1
    }
  }

  if (scrollX != 0) {
    viewRange = endFrame.value - startFrame.value

    startFrame.value += Math.ceil((scrollX * viewRange) / 10)
    endFrame.value += Math.ceil((scrollX * viewRange) / 10)

    startFrame.value = Math.min(
      Math.max(startFrame.value, 0),
      endFrame.value - 1
    )
    endFrame.value = Math.min(
      Math.max(endFrame.value, startFrame.value + 1),
      WorkspaceStore.length
    )

    if (startFrame.value == 0) endFrame.value = viewRange
    if (endFrame.value == WorkspaceStore.length)
      startFrame.value = endFrame.value - viewRange
  }

  startFrame.value = Math.min(Math.max(startFrame.value, 0), endFrame.value - 1)
  endFrame.value = Math.min(
    Math.max(endFrame.value, startFrame.value + 1),
    WorkspaceStore.length
  )

  render()
}

const secondaryColor = '#242424'
const grabColor = '#32a6fc'
const alternateTextColor = '#a7a7a7'

function frameToRelativeX(frame: number): number {
  if (!canvas.value) return -1

  return Math.floor(
    ((frame - startFrame.value) / (endFrame.value - startFrame.value)) *
      canvas.value.width
  )
}

function render() {
  if (!canvas.value) return

  const ctx = canvas.value.getContext('2d')!

  ctx.clearRect(0, 0, canvas.value.width, canvas.value.height)

  // Seperator Bar
  ctx.strokeStyle = secondaryColor
  ctx.lineWidth = 1
  ctx.beginPath()
  ctx.moveTo(0.5, 32.5)
  ctx.lineTo(canvas.value.width, 32)
  ctx.stroke()

  const labelInterval = Math.max(
    Math.floor((endFrame.value - startFrame.value) / 60) * 5,
    1
  )
  const lineInterval = Math.max(
    Math.floor((endFrame.value - startFrame.value) / 60),
    1
  )

  for (let frame = startFrame.value; frame < endFrame.value; frame++) {
    if (frame % lineInterval != 0) continue

    const x = Math.floor(
      ((frame - startFrame.value) / (endFrame.value - startFrame.value)) *
        canvas.value.width
    )

    // Frame bar
    ctx.strokeStyle = alternateTextColor
    ctx.lineWidth = 1
    ctx.beginPath()
    ctx.moveTo(x + 0.5, frame % labelInterval == 0 ? 16 : 24)
    ctx.lineTo(x + 0.5, 32)
    ctx.stroke()

    if (frame % labelInterval != 0) continue

    // Frame numbers
    ctx.fillStyle = alternateTextColor
    ctx.font = '10px JetBrainsMono'
    ctx.fillText(frame.toString(), x + 4, 24)
  }

  // Playhead
  ctx.strokeStyle = grabColor
  ctx.lineWidth = 1
  ctx.beginPath()
  ctx.moveTo(frameToRelativeX(WorkspaceStore.frame) + 0.5, 0)
  ctx.lineTo(frameToRelativeX(WorkspaceStore.frame) + 0.5, canvas.value.height)
  ctx.stroke()
}

watch(
  () => WorkspaceStore.frame,
  () => {
    render()
  }
)

const canvas: Ref<null | HTMLCanvasElement> = ref(null)
const canvasWrapper: Ref<null | HTMLDivElement> = ref(null)

function fixCanvasSize() {
  if (!canvasWrapper.value) return
  if (!canvas.value) return

  canvas.value.width = canvasWrapper.value.offsetWidth
  canvas.value.height = canvasWrapper.value.offsetHeight

  render()
}

onMounted(() => {
  if (!canvasWrapper.value) return

  new ResizeObserver(fixCanvasSize).observe(canvasWrapper.value)

  fixCanvasSize()

  render()
})
</script>

<style scoped>
#component {
  min-height: 18rem;

  border-top: solid 1px var(--secondary);

  display: flex;
  flex-direction: column;
}

#control-bar {
  height: 2rem;

  border-bottom: solid 1px var(--secondary);

  display: flex;
  justify-content: space-between;
  align-items: center;

  box-sizing: border-box;

  padding-left: 0.5rem;
  padding-right: 0.5rem;
}

#canvas-wrapper {
  flex-grow: 1;
}

#canvas {
  display: block;
  position: absolute;
}

.control-bar-group {
  display: flex;
  gap: 0.5rem;
}

#time-input {
  font-size: small;
}

#frame-input {
  font-size: 0.7rem;

  color: var(--alternate-text);

  margin-top: auto;
  margin-bottom: auto;
}

#time-input,
#frame-input {
  width: 3rem;
  background: none;
  padding: 0;
}

#frame-length {
  font-size: 0.7rem;

  color: var(--alternate-text);

  margin: 0;

  width: 3rem;

  text-align: right;

  margin-top: auto;
  margin-bottom: auto;
}

#length {
  font-size: small;

  margin: 0;

  width: 3rem;

  text-align: right;
}
</style>
