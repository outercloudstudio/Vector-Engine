<template>
  <NavBar leftIcon="dataset" leftLink="Projects" />

  <div id="page" @focus="focus">
    <div id="main-split">
      <div id="half-vertical-split">
        <div id="half-horizontal-split">
          <Preview :render="renderPreview" />

          <Sidemenu ref="sideMenu" />
        </div>

        <Timeline />
      </div>

      <div id="side-bar">
        <span
          class="material-symbols-outlined icon-button"
          @click="() => sideMenu?.open('export')"
        >
          save
        </span>

        <span
          class="material-symbols-outlined icon-button"
          @click="() => sideMenu?.open('settings')"
        >
          settings
        </span>

        <span
          class="material-symbols-outlined icon-button"
          @click="() => sideMenu?.open('inspector')"
        >
          colorize
        </span>

        <span
          class="material-symbols-outlined icon-button"
          @click="() => sideMenu?.open('console')"
        >
          terminal
        </span>
      </div>
    </div>
  </div>

  <BasicPopup
    text="Vector Engine needs to access a project folder. Make sure to create one if one does not exist yet!"
    buttonText="Got It"
    :display="displayAccessPopup"
    @confirmed="loadWithPermissions"
  />

  <AlertPopup
    :text="WorkspaceStore.error"
    :display="WorkspaceStore.error != null"
  />
</template>

<script setup lang="ts">
import NavBar from '@/components/NavBar.vue'
import Timeline from '@/components/workspace/Timeline.vue'
import BasicPopup from '@/components/popups/BasicPopup.vue'
import Preview from '@/components/workspace/Preview.vue'
import Sidemenu from '@/components/workspace/Sidemenu.vue'
import AlertPopup from '@/components/popups/AlertPopup.vue'
import { useWorkspaceStore } from '@/stores/WorkspaceStore'
import { ref, onMounted, Ref, onUnmounted, nextTick } from 'vue'

const WorkspaceStore = useWorkspaceStore()

const sideMenu: Ref<null | typeof Sidemenu> = ref(null)

const displayAccessPopup = ref(false)
const renderPreview = ref(false)

async function loadWithPermissions() {
  await WorkspaceStore.loadProjectFromCache()

  displayAccessPopup.value = false
  renderPreview.value = true

  window.addEventListener('focus', focus)
  window.addEventListener('blur', blur)
}

let focused = true

async function focus() {
  focused = true

  const originalFrame = WorkspaceStore.frame

  await WorkspaceStore.loadProjectFromCache()

  await WorkspaceStore.updateFrame(originalFrame)
}

async function blurUpdate() {
  if (focused) return

  const originalFrame = WorkspaceStore.frame

  await WorkspaceStore.loadProjectFromCache()

  await WorkspaceStore.updateFrame(originalFrame)

  setTimeout(blurUpdate, 1000)
}

async function blur() {
  focused = false

  setTimeout(blurUpdate, 1000)
}

onMounted(() => {
  displayAccessPopup.value = !WorkspaceStore.loaded

  if (WorkspaceStore.loaded) loadWithPermissions()
})

onUnmounted(() => {
  window.removeEventListener('focus', focus)
  window.removeEventListener('blur', blur)
})
</script>

<style scoped>
#page {
  width: 100%;
  height: 100vh;

  background-color: var(--main);

  box-sizing: border-box;
  padding-top: 2rem;
}

#main-split {
  display: flex;

  flex-direction: row;

  height: 100%;
}

#side-bar {
  border-left: solid 1px var(--secondary);

  min-width: 2rem;

  display: flex;
  flex-direction: column;

  justify-content: center;
  align-items: center;

  gap: 0.5rem;
}

#half-vertical-split {
  display: flex;
  flex-direction: column;

  flex-grow: 1;

  height: 100%;
}

#half-horizontal-split {
  flex-grow: 1;

  display: flex;
}
</style>
