<template>
  <div id="component">
    <div id="control-bar">
      <div class="control-bar-group">
        <input
          id="time-input"
          v-model.lazy="timeInputBuffer"
          @blur="timeInputBuffer = (<any>$event).target.value"
        />
        <input
          id="frame-input"
          v-model.lazy="frameInputBuffer"
          @blur="frameInputBuffer = (<any>$event).target.value"
        />
      </div>

      <div class="control-bar-group">
        <span class="material-symbols-outlined icon-button"> volume_up </span>
      </div>

      <div class="control-bar-group">
        <span class="material-symbols-outlined icon-button" @click="restart">
          fast_rewind
        </span>

        <span class="material-symbols-outlined icon-button" @click="back">
          skip_previous
        </span>

        <span
          class="material-symbols-outlined icon-button"
          @click="() => (playing ? pause() : play())"
        >
          {{ playing ? 'pause' : 'play_arrow' }}
        </span>

        <span class="material-symbols-outlined icon-button" @click="next">
          skip_next
        </span>
      </div>

      <div class="control-bar-group">
        <span
          id="redo"
          class="material-symbols-outlined icon-button"
          :class="{ looping }"
          @click="loop"
        >
          redo
        </span>

        <span
          class="material-symbols-outlined icon-button"
          @click="createMarker"
        >
          add
        </span>
      </div>

      <div class="control-bar-group">
        <p id="frame-length">{{ WorkspaceStore.length }}</p>
        <p id="length">{{ lengthTime }}</p>
      </div>
    </div>
    <div ref="canvasWrapper" id="canvas-wrapper">
      <canvas
        ref="canvas"
        id="canvas"
        @mousedown="mouseDown"
        @mouseup="mouseUp"
        @mouseleave="mouseUp"
        @mousemove="mouseMove"
        @wheel="scroll"
        @contextmenu.prevent=""
      ></canvas>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, Ref, ref, watch, computed } from 'vue'

import { useWorkspaceStore } from '@/stores/WorkspaceStore'

const WorkspaceStore = useWorkspaceStore()

async function createMarker() {
  WorkspaceStore.createMarker(`Marker`, WorkspaceStore.frame)
}

let playing = ref(false)
let startedPlayingTime = 0
let startedFrame = 0
let audioBufferSource: null | AudioBufferSourceNode = null

async function startAudioPlayback(time: number) {
  const audioBuffer = WorkspaceStore.getAudioBuffer()

  if (!audioBuffer) return

  audioBufferSource = WorkspaceStore.audioContext.createBufferSource()
  audioBufferSource.buffer = audioBuffer
  audioBufferSource.connect(WorkspaceStore.audioContext.destination)
  WorkspaceStore.audioContext.resume()
  audioBufferSource.start(0, time)

  while (WorkspaceStore.audioContext.state == 'suspended') {
    await new Promise<void>(res => {
      setTimeout(() => {
        res()
      }, 1)
    })
  }
}

function stopAudioPlayback() {
  if (!audioBufferSource) return

  audioBufferSource.stop()
}

async function playUpdate() {
  if (!playing.value) return

  const now = Date.now()

  const newFrame =
    Math.floor(((now - startedPlayingTime) / 1000) * WorkspaceStore.frameRate) +
    startedFrame

  await WorkspaceStore.updateFrame(newFrame)

  if (
    WorkspaceStore.frame >= WorkspaceStore.length - 1 ||
    (looping.value &&
      WorkspaceStore.frame >= loopingEnd.value &&
      loopingEnd.value >= 0)
  ) {
    startedPlayingTime = Date.now()
    startedFrame =
      looping.value && loopingStart.value < WorkspaceStore.length
        ? loopingStart.value
        : 0

    stopAudioPlayback()

    await startAudioPlayback(startedFrame / WorkspaceStore.frameRate)
  }

  requestAnimationFrame(playUpdate)
}

async function play() {
  await startAudioPlayback(WorkspaceStore.frame / WorkspaceStore.frameRate)

  playing.value = true
  startedPlayingTime = Date.now()

  if (looping.value) {
    if (
      WorkspaceStore.frame < loopingStart.value &&
      loopingStart.value < WorkspaceStore.length
    ) {
      await WorkspaceStore.updateFrame(loopingStart.value)
    }

    if (WorkspaceStore.frame > loopingEnd.value && loopingEnd.value >= 0) {
      await WorkspaceStore.updateFrame(loopingEnd.value)
    }
  }

  startedFrame = WorkspaceStore.frame

  requestAnimationFrame(playUpdate)
}

function pause() {
  playing.value = false

  stopAudioPlayback()
}

function restart() {
  pause()

  WorkspaceStore.updateFrame(0)
}

function next() {
  pause()

  if (WorkspaceStore.frame >= WorkspaceStore.length - 1) return

  WorkspaceStore.updateFrame(WorkspaceStore.frame + 1)
}

function back() {
  pause()

  if (WorkspaceStore.frame <= 0) return

  WorkspaceStore.updateFrame(WorkspaceStore.frame - 1)
}

const timeInputBuffer = computed({
  get: () => {
    const seconds = WorkspaceStore.frame / WorkspaceStore.frameRate

    const minutes = Math.floor(seconds / WorkspaceStore.frameRate)

    const leftSeconds =
      Math.floor((seconds - minutes * WorkspaceStore.frameRate) * 1000) / 1000

    if (Math.floor(leftSeconds).toString().length == 1)
      return `${minutes}:0${leftSeconds}`

    return `${minutes}:${leftSeconds}`
  },
  set: frame => {
    if (/^(([0-9]+:[0-9]+\.[0-9]{1,3})|([0-9]+:[0-9]+))$/.test(frame)) {
      const minutes = parseInt(frame.split(':')[0])
      const seconds =
        parseFloat(frame.split(':')[1]) + minutes * WorkspaceStore.frameRate
      const frames = Math.floor(seconds * WorkspaceStore.frameRate)

      WorkspaceStore.updateFrame(
        Math.max(Math.min(frames, WorkspaceStore.length - 1), 0)
      )
    } else {
      const originalFrame = WorkspaceStore.frame

      WorkspaceStore.frame = -1

      WorkspaceStore.frame = originalFrame
    }
  },
})

const frameInputBuffer = computed({
  get: () => WorkspaceStore.frame,
  set: (frame: any) => {
    if (/^[0-9]+$/.test(frame)) {
      WorkspaceStore.updateFrame(
        Math.min(Math.max(parseInt(frame), 0), WorkspaceStore.length - 1)
      )
    } else {
      const originalFrame = WorkspaceStore.frame

      WorkspaceStore.frame = -1

      WorkspaceStore.frame = originalFrame
    }
  },
})

const lengthTime = computed(() => {
  const seconds = WorkspaceStore.length / WorkspaceStore.frameRate

  const minutes = Math.floor(seconds / WorkspaceStore.frameRate)

  const leftSeconds =
    Math.floor((seconds - minutes * WorkspaceStore.frameRate) * 1000) / 1000

  if (Math.floor(leftSeconds).toString().length == 1)
    return `${minutes}:0${leftSeconds}`

  return `${minutes}:${leftSeconds}`
})

let mouse = false
let mouseAlt = false

let grabbedScrollbar = false
let grabbedScrollbarOffset = 0

let selectedOriginal = ref(0)
let selectedStart = ref(0)
let selectedEnd = ref(0)
let framesSelected = ref(false)

let highlightedMarker: Ref<string | null> = ref(null)
let heldMarker: Ref<string | null> = ref(null)
let heldMarkerX: Ref<number> = ref(0)
let heldMarkerOffset = 0

function frameToRelativeX(frame: number): number {
  if (!canvas.value) return -1

  return Math.floor(
    ((frame - startFrame.value) / (endFrame.value - startFrame.value)) *
      canvas.value.width
  )
}

function XtoFrame(x: number) {
  if (!canvas.value) return -1

  const factor =
    (x - canvas.value.getBoundingClientRect().left) / canvas.value.width

  return Math.round(
    factor * (endFrame.value - startFrame.value) + startFrame.value
  )
}

function mouseDown(event: MouseEvent) {
  if (!canvas.value) return

  if (event.button == 0) {
    mouse = true

    const relativeY = event.clientY - canvas.value.getBoundingClientRect().top

    if (relativeY >= 58 && relativeY <= 72) {
      const ctx = canvas.value.getContext('2d')!

      for (const marker of WorkspaceStore.markers) {
        if (event.clientX < frameToRelativeX(marker.frame)) continue

        ctx.font = '10px JetBrainsMono'
        const width = ctx.measureText(marker.name).width + 8

        if (event.clientX > frameToRelativeX(marker.frame) + width) continue

        heldMarker.value = marker.id

        heldMarkerOffset = frameToRelativeX(marker.frame) - event.clientX

        highlightedMarker.value = null
      }
    }

    if (
      heldMarker.value == null &&
      event.clientY - canvas.value.getBoundingClientRect().top >=
        canvas.value.height - 8
    ) {
      const relativeX =
        event.clientX - canvas.value.getBoundingClientRect().left

      let scrollBarStart = Math.floor(
        (canvas.value.width * startFrame.value) / WorkspaceStore.length
      )

      const scrollBarWidth = Math.floor(
        (canvas.value.width * (endFrame.value - startFrame.value)) /
          WorkspaceStore.length
      )

      if (
        relativeX < scrollBarStart ||
        relativeX > scrollBarStart + scrollBarWidth
      ) {
        const viewRange = endFrame.value - startFrame.value

        const targetFrame = Math.round(
          (relativeX / canvas.value.width) * WorkspaceStore.length
        )

        startFrame.value = targetFrame - Math.floor(viewRange / 2)
        endFrame.value = targetFrame + Math.ceil(viewRange / 2)

        scrollBarStart = Math.floor(
          (canvas.value.width * startFrame.value) / WorkspaceStore.length
        )
      }

      grabbedScrollbar = true
      grabbedScrollbarOffset = scrollBarStart - relativeX
    }
  } else if (event.button == 2) {
    mouseAlt = true

    selectedOriginal.value = XtoFrame(event.clientX)
    selectedStart.value = XtoFrame(event.clientX)
    framesSelected.value = true
  }

  mouseMove(event)
}

function mouseUp(event: MouseEvent) {
  if (event.button == 0) {
    mouse = false

    grabbedScrollbar = false

    if (heldMarker.value != null) {
      const droppedFrame = XtoFrame(heldMarkerX.value)

      WorkspaceStore.updateMarker(
        heldMarker.value,
        WorkspaceStore.markers.find(marker => marker.id == heldMarker.value)
          .name,
        droppedFrame
      )

      heldMarker.value = null
    }
  } else if (event.button == 2) {
    if (mouseAlt && selectedEnd.value == selectedStart.value) {
      framesSelected.value = false
    }

    mouseAlt = false
  }
}

async function mouseMove(event: MouseEvent) {
  if (!canvas.value) return

  if (mouse) {
    if (!grabbedScrollbar) {
      if (heldMarker.value == null) {
        await WorkspaceStore.updateFrame(
          Math.min(
            Math.max(XtoFrame(event.clientX), 0),
            WorkspaceStore.length - 1
          )
        )
      } else {
        heldMarkerX.value = Math.min(
          Math.max(event.clientX + heldMarkerOffset, frameToRelativeX(0)),
          frameToRelativeX(WorkspaceStore.length - 1)
        )
      }
    } else {
      const relativeX =
        event.clientX - canvas.value.getBoundingClientRect().left

      const newScrollbarPosition = relativeX + grabbedScrollbarOffset

      const newScrollBarFrame = Math.round(
        (newScrollbarPosition / canvas.value.width) * WorkspaceStore.length
      )
      const viewRange = endFrame.value - startFrame.value

      startFrame.value = newScrollBarFrame
      endFrame.value = newScrollBarFrame + viewRange
    }
  } else if (mouseAlt) {
    const frame = XtoFrame(event.clientX)

    if (frame < selectedOriginal.value) {
      selectedEnd.value = selectedOriginal.value
      selectedStart.value = frame
    } else {
      selectedStart.value = selectedOriginal.value
      selectedEnd.value = frame
    }
  } else if (heldMarker.value == null) {
    highlightedMarker.value = null

    const relativeY = event.clientY - canvas.value.getBoundingClientRect().top

    if (relativeY < 58) return

    if (relativeY > 72) return

    const ctx = canvas.value.getContext('2d')!

    for (const marker of WorkspaceStore.markers) {
      if (event.clientX < frameToRelativeX(marker.frame)) continue

      ctx.font = '10px JetBrainsMono'
      const width = ctx.measureText(marker.name).width + 8

      if (event.clientX > frameToRelativeX(marker.frame) + width) continue

      highlightedMarker.value = marker.id
    }
  }
}

let looping = ref(false)
let loopingStart = ref(0)
let loopingEnd = ref(0)

function loop() {
  looping.value = !looping.value

  if (!looping.value) return

  if (!framesSelected.value || selectedStart.value == selectedEnd.value) {
    looping.value = false

    return
  }

  loopingStart.value = selectedStart.value
  loopingEnd.value = selectedEnd.value
}

let startFrame = ref(-5)
let endFrame = ref(59 + 5)

watch(
  () => WorkspaceStore.length,
  () => {
    return (endFrame.value = WorkspaceStore.length - 1 + 5)
  }
)

function scroll(event: any) {
  const scrollX = Math.ceil(event.deltaX / 100)
  const scrollY = Math.ceil(event.deltaY / 100)

  let viewRange = endFrame.value - startFrame.value

  if (scrollY != 0) {
    let zoomAmount =
      (Math.ceil(
        Math.abs(Math.pow(Math.log(viewRange / 59), Math.abs(scrollY)))
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
  }
}

watch(startFrame, () => render())
watch(endFrame, () => render())

const secondaryColor = '#242424'
const grabColor = '#32a6fc'
const alternateGrab = '#17222b'
const textColor = '#d9d9d9'
const alternateTextColor = '#a7a7a7'

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
    ctx.moveTo(x, frame % labelInterval == 0 ? 16 : 24)
    ctx.lineTo(x, 32)
    ctx.stroke()

    if (frame % labelInterval != 0) continue

    // Frame numbers
    ctx.fillStyle = alternateTextColor
    ctx.font = '10px JetBrainsMono'
    ctx.fillText(frame.toString(), x + 4, 24)
  }

  // Volume
  for (let frame = startFrame.value; frame < endFrame.value; frame++) {
    // Frame bar
    ctx.fillStyle = 'rgba(255, 255, 255, 0.1)'

    const size =
      (WorkspaceStore.volumePerFrame[frame] || 0) *
      ((canvas.value.height - 72) / 2)
    ctx.fillRect(
      frameToRelativeX(frame),
      (canvas.value.height - 72) / 2 - size / 2 + 72,
      frameToRelativeX(frame + 1) - frameToRelativeX(frame),
      size
    )
  }

  // Markers
  for (const marker of WorkspaceStore.markers) {
    if (marker.id == heldMarker.value) continue

    ctx.font = '10px JetBrainsMono'
    const width = ctx.measureText(marker.name).width + 8

    if (highlightedMarker.value == marker.id && heldMarker.value == null) {
      ctx.strokeStyle = textColor
      ctx.lineWidth = 1
    }

    ctx.fillStyle = alternateGrab
    ctx.beginPath()
    ctx.roundRect(
      frameToRelativeX(marker.frame),
      58,
      width,
      16,
      [0, 9999, 9999, 9999]
    )
    ctx.fill()
    if (highlightedMarker.value == marker.id && heldMarker.value == null)
      ctx.stroke()

    ctx.fillStyle = textColor
    ctx.fillText(
      marker.name,
      frameToRelativeX(marker.frame) + 4,
      60 + ctx.measureText(marker.name).fontBoundingBoxAscent
    )
  }

  if (heldMarker.value != null) {
    const marker = WorkspaceStore.markers.find(
      marker => marker.id == heldMarker.value
    )

    ctx.font = '10px JetBrainsMono'
    const width = ctx.measureText(marker.name).width + 8

    ctx.strokeStyle = textColor
    ctx.lineWidth = 1
    ctx.fillStyle = alternateGrab
    ctx.beginPath()
    ctx.roundRect(heldMarkerX.value, 58, width, 16, [0, 9999, 9999, 9999])
    ctx.fill()
    ctx.stroke()

    ctx.fillStyle = textColor
    ctx.fillText(
      marker.name,
      heldMarkerX.value + 4,
      60 + ctx.measureText(marker.name).fontBoundingBoxAscent
    )
  }

  // Left Side
  if (startFrame.value < 0) {
    ctx.fillStyle = 'rgba(0, 0, 0, 0.3)'
    ctx.fillRect(0, 0, frameToRelativeX(0), canvas.value.height)
  }

  // Right Side
  if (endFrame.value >= WorkspaceStore.length) {
    ctx.fillStyle = 'rgba(0, 0, 0, 0.3)'
    ctx.fillRect(
      frameToRelativeX(WorkspaceStore.length),
      0,
      frameToRelativeX(endFrame.value) -
        frameToRelativeX(WorkspaceStore.length),
      canvas.value.height
    )
  }

  // Selected Area
  if (framesSelected.value) {
    ctx.fillStyle = 'rgba(255, 255, 255, 0.1)'
    ctx.fillRect(
      frameToRelativeX(selectedStart.value),
      0,
      frameToRelativeX(selectedEnd.value) -
        frameToRelativeX(selectedStart.value),
      canvas.value.height
    )
  }

  // Looping Area
  if (looping.value) {
    ctx.fillStyle = 'rgba(50, 166, 252, 0.3)'
    ctx.fillRect(
      frameToRelativeX(loopingStart.value),
      0,
      frameToRelativeX(loopingEnd.value) - frameToRelativeX(loopingStart.value),
      canvas.value.height
    )
  }

  // Playhead
  ctx.strokeStyle = grabColor
  ctx.lineWidth = 1
  ctx.beginPath()
  ctx.moveTo(frameToRelativeX(WorkspaceStore.frame), 0)
  ctx.lineTo(frameToRelativeX(WorkspaceStore.frame), canvas.value.height)
  ctx.stroke()

  // Scrollbar
  ctx.fillStyle = 'rgba(0, 0, 0, 0.1)'
  ctx.fillRect(0, canvas.value.height, canvas.value.width, -8)

  ctx.fillStyle = '#242424'
  ctx.fillRect(
    Math.floor((canvas.value.width * startFrame.value) / WorkspaceStore.length),
    canvas.value.height,
    Math.floor(
      (canvas.value.width * (endFrame.value - startFrame.value)) /
        WorkspaceStore.length
    ),
    -8
  )
}

watch(
  () => WorkspaceStore.frame,
  () => {
    render()
  }
)

watch(
  () => WorkspaceStore.markers,
  () => {
    render()
  }
)

watch(
  () => WorkspaceStore.volumePerFrame,
  () => {
    render()
  }
)

watch(framesSelected, () => {
  render()
})

watch(selectedStart, () => {
  render()
})

watch(selectedEnd, () => {
  render()
})

watch(looping, () => {
  render()
})

watch(loopingStart, () => {
  render()
})

watch(loopingEnd, () => {
  render()
})

watch(highlightedMarker, () => {
  render()
})

watch(heldMarker, () => {
  render()
})

watch(heldMarkerX, () => {
  render()
})

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

  if (WorkspaceStore.loaded) endFrame.value = WorkspaceStore.length
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
  font-size: x-small;
}

#frame-input {
  font-size: x-small;

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
  font-size: x-small;

  color: var(--alternate-text);

  margin: 0;

  width: 3rem;

  text-align: right;

  margin-top: auto;
  margin-bottom: auto;
}

#length {
  font-size: x-small;

  margin: 0;

  width: 3rem;

  text-align: right;
}

#redo.looping {
  color: var(--grab);
}
</style>
