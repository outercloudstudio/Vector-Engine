<template>
  <NavBarVue leftIcon="home" leftLink="Welcome" />

  <div id="page">
    <!--
    <button @click="chooseEngineFolder">Choose Folder</button>

    <input v-model="newProjectName" />

    <button @click="createProject">Create Project</button>

    <button v-for="project in projects" @click="() => loadProject(project)">{{ project.name }}</button>
    -->

    <div id="flex">
      <div id="project-preview-grid" @click.self="selectedProject = null">
        <ProjectPreviewVue
          v-for="projectName in ProjectsStore.projects"
          :key="projectName"
          :name="projectName"
          :selected="selectedProject == projectName"
          @click="selectedProject = projectName"
          @renamed="renamedProject"
        />
      </div>

      <div id="options-side-bar">
        <span
          id="folder"
          class="material-symbols-outlined icon-button"
          @click="chooseNewProjectFolder"
        >
          folder
        </span>
        <span class="material-symbols-outlined icon-button"> add </span>
        <span class="material-symbols-outlined icon-button"> edit </span>
        <span class="material-symbols-outlined icon-button"> delete </span>
      </div>
    </div>
  </div>

  <PopupVue
    text="Vector Engine needs to access a project folder. Make sure to create one if one does not exist yet!"
    buttonText="Got It"
    :display="displayPopup"
    @confirmed="popupConfirmed"
  />
</template>

<script setup lang="ts">
import { ref, onMounted, Ref } from 'vue'
import router from '@/router'
import {
  hasProjectsFolderPermissions,
  hasProjectsFolder,
  getDirectoryPicker,
  getProjectsFolderPermissions,
  hasPermissions,
  setProjectsFolder,
} from '@/fs'

import NavBarVue from '@/components/NavBar.vue'
import ProjectPreviewVue from '@/components/ProjectPreview.vue'
import PopupVue from '@/components/Popup.vue'

import { useProjectsStore } from '@/stores/ProjectsStore'

const ProjectsStore = useProjectsStore()

const displayPopup = ref(false)
const selectedProject: Ref<null | string> = ref(null)

async function popupConfirmed() {
  if (await hasProjectsFolderPermissions()) {
    displayPopup.value = false

    await ProjectsStore.updateProjects()

    return
  }

  if (await hasProjectsFolder()) {
    await getProjectsFolderPermissions()

    if (!(await hasProjectsFolderPermissions())) return

    await ProjectsStore.updateProjects()
  } else {
    const handle = await getDirectoryPicker()

    if (handle == null) return

    if (!(await hasPermissions(handle))) return

    await setProjectsFolder(handle)

    await ProjectsStore.updateProjects()
  }

  displayPopup.value = false
}

async function chooseNewProjectFolder() {
  const handle = await getDirectoryPicker()

  if (handle == null) return

  if (!(await hasPermissions(handle))) return

  await setProjectsFolder(handle)

  await ProjectsStore.updateProjects()
}

async function renamedProject(previousName: string, newName: string) {
  await ProjectsStore.renameProject(previousName, newName)
}

onMounted(async () => {
  if (!(await hasProjectsFolderPermissions())) displayPopup.value = true
})

// const ProjectStore = useProjectStore()
// const SettingsStore = useSettingsStore()

// let projects = ref([])

// let newProjectName = ref('New Project')

// async function chooseEngineFolder() {
//   const dirHandle = getDirectoryPicker()

//   if (!(await getFilePermissions(dirHandle))) return

//   await set('engine-folder', dirHandle)

//   await loadProjects()
// }

// async function loadProject(dir: any) {
//   await ProjectStore.setupProject(dir)

//   router.push({ name: 'Editor' })
// }

// async function loadProjects() {
//   const dbKeys = await keys()

//   if (!dbKeys.includes('engine-folder')) return

//   projects.value = []

//   const dir = await get('engine-folder')

//   if (!(await getPermissions(dir))) return

//   await SettingsStore.load(dir)

//   for await (const entry of dir.values()) {
//     if (await isDirAProject(entry)) projects.value.push(entry)
//   }
// }

// async function createProject() {
//   const dbKeys = await keys()

//   if (!dbKeys.includes('engine-folder')) return

//   const dir = await get('engine-folder')

//   if (!(await getPermissions(dir))) return

//   if (await doesFolderExistOnFolderHandle(newProjectName.value, dir)) return

//   const projectDir = await dir.getDirectoryHandle(newProjectName.value, { create: true })

//   await ProjectStore.createProject(projectDir)

//   router.push({ name: 'Editor' })
// }

// onMounted(async () => {
//   await loadProjects()
// })
</script>

<style scoped>
#page {
  width: 100%;
  height: 100vh;

  padding-top: 2rem;
  box-sizing: border-box;

  background-color: var(--main);

  overflow: hidden;
}

#flex {
  display: flex;
  flex-direction: row;
  align-items: flex-start;

  min-height: 100%;
}

#project-preview-grid {
  display: flex;
  flex-wrap: wrap;

  justify-content: space-evenly;
  align-items: flex-start;

  min-height: calc(100vh - 2rem);

  overflow-y: auto;

  padding-top: 0.5rem;
  padding-left: 0.5rem;
  box-sizing: border-box;

  width: 100%;
}

#options-side-bar {
  min-width: 2rem;
  height: calc(100vh - 2rem);

  border-left: 1px solid var(--secondary);

  display: flex;
  flex-direction: column;

  justify-content: center;
  align-items: center;
}

#options-side-bar > * {
  margin-bottom: 0.25rem;
  margin-top: 0.25rem;
}

#folder {
  margin-bottom: 1rem;
}
</style>
