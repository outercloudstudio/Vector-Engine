<template>
  <NavBarVue leftIcon="dataset" leftLink="Projects" />

  <div id="page">
    <div id="main-split">
      <div id="half-vertical-split">
        <div id="half-horizontal-split">
          <div ref="previewWrapper" id="preview-wrapper">
            <canvas ref="preview" id="preview"></canvas>
          </div>

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
import { useWorkspaceStore } from '@/stores/WorkspaceStore'
import { ref, onMounted, Ref } from 'vue'

const WorkspaceStore = useWorkspaceStore()

const displayAccessPopup = ref(false)

async function loadWithPermissions() {
  await WorkspaceStore.loadProjectFromCache()

  displayAccessPopup.value = false

  preview.value
    ?.getContext('2d')
    ?.drawImage((await WorkspaceStore.render())!, 0, 0, 192, 108)
}

const preview: Ref<null | HTMLCanvasElement> = ref(null)
const previewWrapper: Ref<null | HTMLDivElement> = ref(null)

function fixPreviewSize() {
  if (!previewWrapper.value) return
  if (!preview.value) return

  preview.value.width = previewWrapper.value.offsetWidth
  preview.value.height = previewWrapper.value.offsetHeight
}

onMounted(() => {
  if (!previewWrapper.value) return

  new ResizeObserver(fixPreviewSize).observe(previewWrapper.value)

  fixPreviewSize()

  displayAccessPopup.value = !WorkspaceStore.loaded

  if (WorkspaceStore.loaded) loadWithPermissions()
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

#preview-wrapper {
  background-color: black;

  flex-grow: 1;

  margin: 0;
  padding: 0;

  max-height: 100%;
}

#preview {
  display: block;

  margin: 0;

  position: absolute;
}

#side-menu {
  min-width: 14rem;

  border-left: solid 1px var(--secondary);
}
</style>
