import { defineStore } from 'pinia'
import { ref, computed, toRaw, watch, Ref } from 'vue'
import { getProjectsFolder, getFolders } from '@/fs'

export const useProjectsStore = defineStore('ProjectsStore', () => {
  const projects: Ref<any[]> = ref([])

  async function updateProjects() {
    projects.value = []

    for (const potentialProject of await getFolders(
      await getProjectsFolder()
    )) {
      projects.value.push(potentialProject.name)
    }
  }

  async function renameProject(
    previousName: string,
    newName: string,
    handle: any
  ) {
    const projectIndex = projects.value.indexOf(previousName)

    if (projectIndex == -1) return

    projects.value[projectIndex] = newName
  }

  return {
    projects,
    updateProjects,
    renameProject,
  }
})
