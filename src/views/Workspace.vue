<template>
  <NavBarVue leftIcon="dataset" leftLink="Projects" />

  <div id="page" @focus="focus">
    <div id="main-split">
      <div id="half-vertical-split">
        <div id="half-horizontal-split">
          <PreviewVue :render="renderPreview" />

          <div id="side-menu"></div>
        </div>

        <TimelineVue />
      </div>

      <div id="side-bar">
        <span class="material-symbols-outlined icon-button"> save </span>
        <span class="material-symbols-outlined icon-button"> settings </span>
        <span class="material-symbols-outlined icon-button"> colorize </span>
      </div>
    </div>
  </div>

  <BasicPopupVue
    text="Vector Engine needs to access a project folder. Make sure to create one if one does not exist yet!"
    buttonText="Got It"
    :display="displayAccessPopup"
    @confirmed="loadWithPermissions"
  />
</template>

<script setup lang="ts">
import NavBarVue from '@/components/NavBar.vue'
import TimelineVue from '@/components/workspace/Timeline.vue'
import BasicPopupVue from '@/components/popups/BasicPopup.vue'
import PreviewVue from '@/components/workspace/Preview.vue'
import { useWorkspaceStore } from '@/stores/WorkspaceStore'
import { ref, onMounted, Ref, onUnmounted } from 'vue'

const WorkspaceStore = useWorkspaceStore()

const displayAccessPopup = ref(false)
const renderPreview = ref(false)

async function loadWithPermissions() {
  await WorkspaceStore.loadProjectFromCache()

  displayAccessPopup.value = false
  renderPreview.value = true
}

async function focus() {
  const originalFrame = WorkspaceStore.frame

  await WorkspaceStore.loadProjectFromCache()
  await WorkspaceStore.updateFrame(originalFrame)
}

onMounted(() => {
  displayAccessPopup.value = !WorkspaceStore.loaded

  if (WorkspaceStore.loaded) loadWithPermissions()

  window.addEventListener('focus', focus)
})

onUnmounted(() => {
  window.removeEventListener('focus', focus)
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

#side-menu {
  min-width: 14rem;

  border-left: solid 1px var(--secondary);
}
</style>
