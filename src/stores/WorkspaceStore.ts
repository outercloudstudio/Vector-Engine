import { defineStore } from 'pinia'
import { Ref, ref } from 'vue'
import { getProjectFolder, cacheProjectFolder } from '@/fs'
import { Engine } from '@/engine/engine'
import { Runtime } from '@/Runtime'

export const useWorkspaceStore = defineStore('WorkspaceStore', () => {
  let projectFolder: FileSystemDirectoryHandle | undefined = undefined
  let engine: Engine | undefined = undefined
  let loaded: Ref<boolean> = ref(false)
  let frame: Ref<number> = ref(0)
  let length: Ref<number> = ref(60)

  async function loadProject(name: string) {
    projectFolder = (await getProjectFolder(name)) || undefined

    if (!projectFolder) return

    cacheProjectFolder(projectFolder)

    const runtime = new Runtime(projectFolder)

    engine = new Engine(runtime)
    await engine.load()

    loaded.value = true
  }

  async function loadProjectFromCache() {
    await loadProject('')
  }

  async function render() {
    return await engine?.render()
  }

  return { loadProject, loadProjectFromCache, render, loaded, frame, length }
})
