<template>
  <ErrorWindow
    :message="ProjectStore.error"
    v-show="ProjectStore.error != null"
  />

  <ContextMenu ref="contextMenu" @newMarker="newMarker" />

  <div id="main">
    <div id="motor">
      <div id="canvas-container">
        <canvas width="1920" height="1080" id="canvas" ref="canvas" />
      </div>

      <SideMenu
        ref="sideMenu"
        :canvas="canvas"
        :currentFrame="currentFrame"
        v-model:selectedMarker="selectedMarker"
        @updateFrame="updateFrame"
      />
    </div>

    <ActionBar
      :isPlaying="isPlaying"
      :currentFrame="currentFrame"
      v-model:playbackSpeed="playbackSpeed"
      @updateFrame="updateFrame"
      @pause="pause"
      @next="next"
      @prev="prev"
      @reset="reset"
    />

    <TimeLine
      ref="timeline"
      :events="events"
      :currentFrame="currentFrame"
      v-model:selectedMarker="selectedMarker"
      @updateFrame="updateFrame"
      @context="(x, y) => contextMenu.open(x, y)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import router from '@/router'
import { Engine } from '@/engine'
import SideMenu from '@/components/SideMenu.vue'
import ActionBar from '@/components/ActionBar.vue'
import TimeLine from '@/components/TimeLine.vue'
import ErrorWindow from '@/components/ErrorWindow.vue'
import ContextMenu from '@/components/ContextMenu.vue'
import { useProjectStore } from '@/stores/ProjectStore'
import { useSettingsStore } from '@/stores/SettingsStore'

const ProjectStore = useProjectStore()
const SettingsStore = useSettingsStore()

let contextMenu = ref(null)
let sideMenu = ref(null)

let canvas = ref(null)
let timeline = ref(null)

let currentFrame = ref(0)
let currentlyRenderedFrame = 0
let shouldReload = false

let events = ref([])

let selectedMarker = ref('')

const clamp = (num, min, max) => Math.min(Math.max(num, min), max)

let playbackSpeed = ref(1)
let isPlaying = ref(false)
let timeSinceStart = 0
let startFrame = 0

let setup = false

watch(selectedMarker, (val, prevVal) => {
  if (val == '') {
    sideMenu.value.closeMenu()
  } else {
    sideMenu.value.forceOpenMenu('marker')
  }
})

watch(currentFrame, async (frame, prevFrame) => {
  const currentTime = ProjectStore.masterAudioTrack.currentTime
  const frameTime = frame / ProjectStore.frameRate
  const diff = Math.abs(currentTime - frameTime)

  if (diff >= 0.1) {
    ProjectStore.masterAudioTrack.currentTime = frameTime

    if (isPlaying.value && ProjectStore.masterAudioTrack.paused)
      ProjectStore.masterAudioTrack.play()

    //console.log(`Audio time ${currentTime} Frame time ${frameTime} Difference ${diff}`)
  }
})

watch(
  () => ProjectStore.vfsVersion,
  async (code, prevCode) => {
    if (!setup) return

    shouldReload = true

    events.value = await engine.inference(ProjectStore.animationLength)
  }
)

watch(
  () => ProjectStore.animationLength,
  (length, prevLength) => {
    if (length < prevLength) {
      currentFrame.value = length - 1
    } else {
      render()
    }
  }
)

async function nextPlayFrame() {
  if (!isPlaying.value) return

  const currentTime = new Date().getTime()

  const ms = currentTime - timeSinceStart

  const frame = Math.round((ms / 1000) * ProjectStore.frameRate)

  currentFrame.value = (startFrame + frame) % ProjectStore.animationLength

  window.requestAnimationFrame(() => {
    nextPlayFrame()
  })
}

async function next() {
  if (isPlaying.value) {
    ProjectStore.masterAudioTrack.pause()

    isPlaying.value = false
  }

  if (currentFrame.value < ProjectStore.animationLength - 1)
    currentFrame.value++
}

async function prev() {
  if (isPlaying.value) {
    ProjectStore.masterAudioTrack.pause()

    isPlaying.value = false
  }

  if (currentFrame.value > 0) currentFrame.value--
}

async function pause() {
  if (isPlaying.value) {
    ProjectStore.masterAudioTrack.pause()

    isPlaying.value = false
  } else {
    isPlaying.value = true

    ProjectStore.masterAudioTrack.play()

    timeSinceStart = new Date().getTime()
    startFrame = currentFrame.value

    nextPlayFrame()
  }
}

async function reset() {
  if (isPlaying.value) {
    ProjectStore.masterAudioTrack.pause()

    isPlaying.value = false
  }

  currentFrame.value = 0

  shouldReload = true
}

function newMarker(x) {
  ProjectStore.newMarker(timeline.value.screenXToFrame(x))
}

async function updateFrame(frame) {
  if (isPlaying.value) {
    startFrame += frame - currentFrame.value
  } else {
    currentFrame.value = frame
  }
}

const engine = new Engine()

function reloadContext() {
  engine.reloadContext(
    ProjectStore.initialScene,
    ProjectStore.initialSceneName,
    ProjectStore.frameRate,
    ProjectStore.scenes,
    ProjectStore.markers,
    ProjectStore.getAssets()
  )
}

async function render() {
  const ctx = canvas.value.getContext('2d')

  ctx.fillStyle = 'white'
  ctx.fillRect(0, 0, 1920, 1080)

  await engine.render(ctx)
}

async function renderLoop() {
  const cf = currentFrame.value

  if (cf == currentlyRenderedFrame && !shouldReload) {
    window.requestAnimationFrame(() => {
      renderLoop()
    })

    return
  }

  shouldReload = false

  if (cf <= currentlyRenderedFrame) {
    reloadContext()

    await engine.stepFrame()

    for (let i = 0; i < cf; i++) {
      await engine.stepFrame()
    }
  } else {
    for (let i = 0; i < cf - currentlyRenderedFrame; i++) {
      await engine.stepFrame()
    }
  }

  currentlyRenderedFrame = cf

  await render()

  window.requestAnimationFrame(() => {
    renderLoop()
  })
}

onMounted(async () => {
  await SettingsStore.recover()

  if (!(await ProjectStore.recover())) router.push({ name: 'Projects' })

  reloadContext()

  await engine.stepFrame()

  await render()

  events.value = await engine.inference(ProjectStore.animationLength)

  await renderLoop()

  window.addEventListener('focus', async () => {
    await SettingsStore.reload()
    if (!(await ProjectStore.reload())) router.push({ name: 'Projects' })
  })

  setup = true
})
</script>

<style scoped>
#main {
  left: 1rem;
  right: 1rem;
  top: 1rem;
  bottom: 1rem;

  position: absolute;

  display: flex;
  flex-direction: column;
}

#motor {
  display: flex;

  justify-content: space-between;

  flex-grow: 1;
}

#code-container {
  flex-grow: 2;

  width: 8rem;
  min-width: 0;

  background: var(--darker);
}

#canvas-container {
  flex-grow: 1;

  display: flex;
  justify-content: center;
  align-items: center;
}

#canvas {
  width: 48rem;
  height: 27rem;

  display: table;

  background: #ffffff;
}
</style>
