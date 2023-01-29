<template>
  <div
    v-show="openMenu != 'none'"
    id="sidemenu"
    :class="{ large: openMenu == 'console' }"
  >
    <div v-if="openMenu == 'export'" class="menu">
      <input placeholder="Export name" v-model="exportName" />

      <div id="alert" v-if="exportNameValidationResult.status != 'none'">
        <span class="material-symbols-outlined icon-button" id="alert-symbol">
          error
        </span>
        <p id="alert-message" v-if="exportNameValidationResult.message != ''">
          {{ exportNameValidationResult.message }}
        </p>
      </div>

      <button
        :class="{ error: exportNameValidationResult.status == 'error' }"
        :disabled="
          exportInProgress || exportNameValidationResult.status == 'error'
        "
        @click="runExport"
      >
        Export
      </button>

      <div id="loading-bar-wrapper" v-if="exportInProgress">
        <div id="loading-bar" :style="{ width: exportProgress + '%' }"></div>
      </div>
    </div>

    <div v-if="openMenu == 'settings'" class="menu">
      <p class="label">Volume</p>
      <input
        id="volume-slider"
        type="range"
        min="0"
        max="100"
        v-model="volumeBuffer"
      />

      <p class="label">Audio Inference</p>
      <Toggle v-model="inferenceAudio" />

      <p class="label">Scene Inference</p>
      <Toggle v-model="inferenceScenes" />
    </div>

    <div v-if="openMenu == 'inspector'" class="menu">
      <input
        placeholder="Marker name"
        v-model.lazy="markerNameInputBuffer"
        @blur="markerNameInputBuffer = (<any>$event).target.value"
      />

      <p class="label">Frame</p>
      <input
        placeholder="Marker frame"
        v-model.lazy="markerFrameInputBuffer"
        @blur="markerFrameInputBuffer = (<any>$event).target.value"
      />

      <span
        id="delete-marker-button"
        class="material-symbols-outlined icon-button"
        @click="deleteMarker"
        >delete</span
      >
    </div>

    <div
      v-show="openMenu == 'console'"
      ref="consoleMenu"
      id="console"
      class="menu"
    >
      <div id="console-label">
        <p>Console</p>

        <span
          class="material-symbols-outlined icon-button"
          @click="() => (WorkspaceStore.errors = [])"
          >delete_forever</span
        >
      </div>

      <div ref="logs" id="logs">
        <p v-for="error in WorkspaceStore.errors">{{ error }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import Toggle from '@/components/Toggle.vue'
import { Runtime } from '@/Runtime'
import { useWorkspaceStore } from '@/stores/WorkspaceStore'
import { Engine } from '@/engine/Engine'
import { computed, onMounted, Ref, ref, watch } from 'vue'

const WorkspaceStore = useWorkspaceStore()

function deleteMarker() {
  if (!WorkspaceStore.selectedMarker) return

  WorkspaceStore.deleteMarker(WorkspaceStore.selectedMarker)
}

let markerName = ref('')
let markerFrame = ref(0)

watch(
  () => WorkspaceStore.selectedMarker,
  markerId => {
    if (markerId == null) {
      if (openMenu.value == 'inspector') open('inspector')

      return
    }

    const marker = WorkspaceStore.markers.find(marker => marker.id == markerId)

    markerName.value = marker.name
    markerFrame.value = marker.frame
  }
)

watch(
  () => WorkspaceStore.markers,
  () => {
    const marker = WorkspaceStore.markers.find(
      marker => marker.id == WorkspaceStore.selectedMarker
    )

    if (!marker) {
      if (openMenu.value == 'inspector') open('inspector')

      WorkspaceStore.selectedMarker = null

      return
    }

    markerName.value = marker.name
    markerFrame.value = marker.frame
  }
)

const markerNameInputBuffer = computed({
  get: () => {
    return markerName.value
  },
  set: passedMarkerName => {
    if (passedMarkerName != '') {
      markerName.value = passedMarkerName

      if (WorkspaceStore.selectedMarker == null) return

      WorkspaceStore.updateMarker(
        WorkspaceStore.selectedMarker,
        markerName.value,
        markerFrame.value
      )
    } else {
      const originalName = markerName.value

      markerName.value = ''

      markerName.value = originalName
    }
  },
})

const markerFrameInputBuffer = computed({
  get: () => {
    return markerFrame.value.toString()
  },
  set: passedMarkerFrame => {
    if (/^[0-9]+$/.test(passedMarkerFrame)) {
      const readMarkerFrame = Math.min(
        Math.max(parseInt(passedMarkerFrame), 0),
        WorkspaceStore.length - 1
      )

      markerFrame.value = readMarkerFrame

      if (WorkspaceStore.selectedMarker == null) return

      WorkspaceStore.updateMarker(
        WorkspaceStore.selectedMarker,
        markerName.value,
        markerFrame.value
      )
    } else {
      const originalFrame = markerFrame.value
      markerFrame.value = -1
      markerFrame.value = originalFrame
    }
  },
})

const inferenceAudio = computed({
  get: () => WorkspaceStore.inferenceAudio,
  set: (inference: boolean) => {
    WorkspaceStore.updateInferenceAudio(inference)
  },
})

const inferenceScenes = computed({
  get: () => WorkspaceStore.inferenceScenes,
  set: (inference: boolean) => {
    WorkspaceStore.updateInferenceScenes(inference)
  },
})

const volumeBuffer = computed({
  get: () => WorkspaceStore.volume * 100,
  set: (volume: number) => {
    WorkspaceStore.updateVolume(volume / 100)
  },
})

const exportName = ref('')

const exportNameValidationResult = computed(() => {
  if (exportName.value == '')
    return { status: 'error', message: 'You must provide an export name' }

  return { status: 'none' }
})

const exportInProgress = ref(false)
const exportProgress = ref(0)

async function runExport() {
  exportInProgress.value = true
  exportProgress.value = 0

  if (!WorkspaceStore.projectFolder) return

  const runtime = new Runtime(WorkspaceStore.projectFolder)

  const engine = new Engine(runtime, WorkspaceStore.markers)
  await engine.load()

  const exportsFolder = await WorkspaceStore.projectFolder.getDirectoryHandle(
    'Exports'
  )

  const exportFolder = await exportsFolder.getDirectoryHandle(
    exportName.value,
    {
      create: true,
    }
  )

  const frameDigits = engine.length.toString.length

  for (let frame = 0; frame < engine.length; frame++) {
    await engine.next()
    const render = await engine.render()

    const renderBlob = await new Promise(res => {
      render.toBlob(blob => {
        res(blob)
      })
    })

    const frameFileHandle = await exportFolder.getFileHandle(
      `frame_${frame.toString().padStart(frameDigits, '0')}.png`,
      { create: true }
    )

    // @ts-ignore
    const writable = await frameFileHandle.createWritable()
    await writable.write(renderBlob)
    await writable.close()

    exportProgress.value = (frame / engine.length) * 100
  }

  exportInProgress.value = false
}

const openMenu = ref('none')

function open(menu: string) {
  if (menu == openMenu.value) {
    openMenu.value = 'none'

    return
  }

  if (menu == 'inspector' && !WorkspaceStore.selectedMarker) return

  openMenu.value = menu
}

defineExpose({
  open,
})

const consoleMenu: Ref<null | HTMLDivElement> = ref(null)
const logs: Ref<null | HTMLDivElement> = ref(null)

function fixLogsSize() {
  if (!consoleMenu.value) return
  if (!logs.value) return

  logs.value.style.height = `calc(${consoleMenu.value.offsetHeight}px - 2rem)`
}

onMounted(() => {
  if (!consoleMenu.value) return

  new ResizeObserver(fixLogsSize).observe(consoleMenu.value)

  fixLogsSize()
})
</script>

<style scoped>
.menu > input {
  width: calc(100% - 0.5rem);
}

#delete-marker-button {
  margin-left: auto;
}

#console {
  height: 100%;
}

#console-label {
  display: flex;
  justify-content: space-between;
  align-items: center;

  width: 100%;
}

#console-label > p {
  margin: 0;

  font-size: x-small;
}

#console-label > span {
  scale: 0.8;
}

#logs {
  overflow-y: scroll;

  max-width: 31rem;
  width: 31rem;

  position: absolute;

  margin-top: 2rem;
}

#logs > p {
  font-size: x-small;
  border-bottom: 1px solid var(--secondary);
  margin-bottom: 0.5rem;
  padding-bottom: 0.5rem;
  white-space: pre-wrap;
}

#sidemenu {
  min-width: 14rem;
  max-width: 14rem;

  border-left: solid 1px var(--secondary);

  padding: 0.5rem;
  box-sizing: border-box;
}

#sidemenu.large {
  min-width: 32rem;
  max-width: 32rem;
}

.menu {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.5rem;
}

#volume-slider {
  appearance: none;
  height: 0.4rem;
  background-color: var(--secondary);
  outline: none;

  width: 100%;
  margin: 0;
  padding: 0;
}

.label {
  font-size: x-small;
  margin: 0;
  padding: 0;
}

#volume-slider::-webkit-slider-thumb {
  appearance: none;
  width: 0.7rem;
  height: 0.7rem;
  background: var(--grab);
  cursor: pointer;
  border-radius: 50%;
}

input {
  padding: 0.25rem;
  background-color: var(--secondary);
  font-size: x-small;
}

button {
  background-color: var(--secondary);
  color: var(--text);
  padding-left: 0.5rem;
  padding-right: 0.5rem;
  padding-top: 0.25rem;
  padding-bottom: 0.25rem;

  font-size: x-small;

  transition: scale 100ms ease-in-out;
}

button.error {
  opacity: 0.5;
}

button:hover:not(.error) {
  scale: 1.1;
}

#loading-bar-wrapper {
  background-color: var(--secondary);

  width: 100%;
  height: 1rem;
}

#loading-bar {
  background-color: var(--grab);

  height: 100%;
}

#alert {
  display: flex;
  align-items: center;

  width: 100%;
}

#alert-message {
  color: var(--grab);

  font-size: x-small;

  margin: 0;
}

#alert-symbol {
  scale: 0.5;

  height: 1.5rem;

  color: var(--grab);
}
</style>
