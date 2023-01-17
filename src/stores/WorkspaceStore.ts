import { defineStore } from 'pinia'
import { Ref, ref, watch, computed } from 'vue'
import { getProjectFolder, cacheProjectFolder } from '@/fs'
import { Engine } from '@/engine/engine'
import { Runtime } from '@/Runtime'

export const useWorkspaceStore = defineStore('WorkspaceStore', () => {
  let projectFolder: Ref<FileSystemDirectoryHandle | undefined> = ref(undefined)
  let engine: Ref<Engine | undefined> = ref(undefined)
  let loaded: Ref<boolean> = ref(false)
  let frame: Ref<number> = ref(0)
  let length: Ref<number> = ref(60)
  let reloadCount: Ref<number> = ref(0)

  async function loadProject(name: string) {
    projectFolder.value = (await getProjectFolder(name)) || undefined

    if (!projectFolder.value) return

    cacheProjectFolder(projectFolder.value)

    const runtime = new Runtime(projectFolder.value)

    engine.value = new Engine(runtime)
    await engine.value.load()

    length.value = engine.value.length
    frame.value = 0

    reloadCount.value++

    loaded.value = true
  }

  async function loadProjectFromCache() {
    await loadProject('')
  }

  async function render() {
    return await engine.value?.render()
  }

  async function updateFrame(frameNumber: number) {
    if (!engine.value) return

    if (frameNumber == frame.value) return

    if (frameNumber < frame.value) {
      await engine.value.reloadContext()

      for (let frameOffset = 0; frameOffset <= frameNumber; frameOffset++) {
        await engine.value.next()
      }

      frame.value = frameNumber

      return
    }

    for (
      let frameOffset = frame.value;
      frameOffset < frameNumber;
      frameOffset++
    ) {
      await engine.value.next()
    }

    frame.value = frameNumber
  }

  const frameRate = computed(() => engine.value?.frameRate || 60)

  return {
    loadProject,
    loadProjectFromCache,
    render,
    loaded,
    frame,
    length,
    reloadCount,
    updateFrame,
    frameRate,
    projectFolder,
  }
})
