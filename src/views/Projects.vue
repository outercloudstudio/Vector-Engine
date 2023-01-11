<template>
  <NavBarVue leftIcon="home" leftLink="Welcome" />

  <div id="page">
    <div id="flex">
      <div id="project-preview-grid" @click.self="selectedProject = null">
        <ProjectPreviewVue
          v-for="project in ProjectsStore.projects"
          :title="project"
          :selected="selectedProject == project"
          @click="() => projectClicked(project)"
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
        <span class="material-symbols-outlined icon-button" @click="newProject">
          add
        </span>
        <span
          class="material-symbols-outlined icon-button"
          @click="deleteProjectButton"
        >
          delete
        </span>
      </div>
    </div>
  </div>

  <BasicPopupVue
    text="Vector Engine needs to access a project folder. Make sure to create one if one does not exist yet!"
    buttonText="Got It"
    :display="displayPopup"
    @confirmed="popupConfirmed"
  />

  <InputPopupVue
    text="Create a new project!"
    placeholder="Enter a project name..."
    buttonText="Submit"
    :display="displayNewProjectPopup"
    :validation="projectNameValidation"
    @confirmed="newProjectPopupConfirmed"
    @cancelled="newProjectPopupCancelled"
  />

  <ConfirmPopupVue
    buttonText="Got It"
    :text="`Are you sure you want to delete '${projectToDeleteName}'?`"
    :display="displayDeleteProjectPopup"
    @confirmed="deleteProjectPopupConfirmed"
    @cancelled="deleteProjectPopupCancelled"
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
  createProject,
  deleteProject,
} from '@/fs'

import NavBarVue from '@/components/NavBar.vue'
import ProjectPreviewVue from '@/components/ProjectPreview.vue'
import BasicPopupVue from '@/components/popups/BasicPopup.vue'
import InputPopupVue from '@/components/popups/InputPopup.vue'
import ConfirmPopupVue from '@/components/popups/ConfirmPopup.vue'

import { useProjectsStore } from '@/stores/ProjectsStore'
import { useWorkspaceStore } from '@/stores/WorkspaceStore'

const ProjectsStore = useProjectsStore()
const WorkspaceStore = useWorkspaceStore()

const displayPopup = ref(false)
const displayNewProjectPopup = ref(false)
const selectedProject: Ref<null | string> = ref(null)
const displayDeleteProjectPopup = ref(false)
const projectToDeleteName = ref('')

let lastProjectClickTime: null | number = null

async function projectClicked(project: string) {
  const now = Date.now()

  if (
    lastProjectClickTime != null &&
    selectedProject.value == project &&
    now - lastProjectClickTime < 300
  ) {
    await WorkspaceStore.loadProject(project)

    router.push({ name: 'Workspace' })
  }

  lastProjectClickTime = now

  selectedProject.value = project
}

function deleteProjectButton() {
  if (!selectedProject.value) return

  projectToDeleteName.value = selectedProject.value

  displayDeleteProjectPopup.value = true
}

async function deleteProjectPopupConfirmed() {
  await deleteProject(projectToDeleteName.value)

  await ProjectsStore.updateProjects()

  displayDeleteProjectPopup.value = false
}

function deleteProjectPopupCancelled() {
  displayDeleteProjectPopup.value = false
}

function newProject() {
  displayNewProjectPopup.value = true
}

function projectNameValidation(name: string): {
  status: string
  message?: string
} {
  if (name == '')
    return {
      status: 'error',
      message: "Project name can't be empty",
    }

  if (ProjectsStore.projects.includes(name))
    return {
      status: 'error',
      message: 'A project with that name already exists!',
    }

  return {
    status: 'none',
  }
}

async function newProjectPopupConfirmed(name: string) {
  await createProject(name)

  await ProjectsStore.updateProjects()

  displayNewProjectPopup.value = false
}

function newProjectPopupCancelled() {
  displayNewProjectPopup.value = false
}

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

onMounted(async () => {
  if (!(await hasProjectsFolderPermissions())) displayPopup.value = true
})
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
