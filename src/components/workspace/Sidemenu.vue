<template>
  <div id="sidemenu" v-if="openMenu != 'none'">
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
  </div>
</template>

<script setup lang="ts">
import Toggle from '@/components/Toggle.vue'
import { Runtime } from '@/Runtime'
import { useWorkspaceStore } from '@/stores/WorkspaceStore'
import { Engine } from '@/engine/Engine'
import { computed, ref } from 'vue'

const WorkspaceStore = useWorkspaceStore()

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

  openMenu.value = menu
}

defineExpose({
  open,
})
</script>

<style scoped>
#sidemenu {
  min-width: 14rem;
  max-width: 14rem;

  border-left: solid 1px var(--secondary);

  padding: 0.5rem;
  box-sizing: border-box;
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
