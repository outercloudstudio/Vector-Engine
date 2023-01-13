import { defineStore } from 'pinia'
import { Ref, ref } from 'vue'
import { getProjectFolder } from '@/fs'
import { Engine } from '@/engine/engine'
import { Runtime } from '@/Runtime'

export const useWorkspaceStore = defineStore('WorkspaceStore', () => {
  let projectFolder: FileSystemDirectoryHandle | undefined = undefined
  let engine: Engine | undefined = undefined

  async function loadProject(name: string) {
    projectFolder = await getProjectFolder(name)

    if (!projectFolder) return

    const runtime = new Runtime(projectFolder)

    engine = new Engine(runtime)
    await engine.load()
  }

  async function render() {
    return await engine?.render()
  }

  return { loadProject, render }
})
