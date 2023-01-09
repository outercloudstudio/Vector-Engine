import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getProjectFolder } from '@/fs'

export const useEditorStore = defineStore('EditorStore', () => {
  const projectFolder = ref(null)

  async function loadProject(name: string) {
    projectFolder.value = await getProjectFolder(name)
  }

  return { loadProject }
})
