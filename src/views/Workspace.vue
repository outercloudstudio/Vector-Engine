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

        <div id="timeline">
          <div id="control-bar"></div>
          <div ref="timelineCanvasWrapper" id="timeline-canvas-wrapper">
            <canvas ref="timelineCanvas" id="timeline-canvas"></canvas>
          </div>
        </div>
      </div>

      <div id="side-bar">
        <span class="material-symbols-outlined icon-button"> save </span>
        <span class="material-symbols-outlined icon-button"> settings </span>
        <span class="material-symbols-outlined icon-button"> colorize </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import NavBarVue from '@/components/NavBar.vue'
import { ref, onMounted, Ref } from 'vue'

const preview: Ref<null | HTMLCanvasElement> = ref(null)
const previewWrapper: Ref<null | HTMLDivElement> = ref(null)

const timelineCanvas: Ref<null | HTMLCanvasElement> = ref(null)
const timelineCanvasWrapper: Ref<null | HTMLDivElement> = ref(null)

function fixPreviewSize() {
  if (!previewWrapper.value) return
  if (!preview.value) return

  preview.value.width = previewWrapper.value.offsetWidth
  preview.value.height = previewWrapper.value.offsetHeight
}

function fixTimelineCanvasSize() {
  if (!timelineCanvasWrapper.value) return
  if (!timelineCanvas.value) return

  timelineCanvas.value.width = timelineCanvasWrapper.value.offsetWidth
  timelineCanvas.value.height = timelineCanvasWrapper.value.offsetHeight
}

onMounted(() => {
  if (!previewWrapper.value) return
  if (!timelineCanvasWrapper.value) return

  new ResizeObserver(fixPreviewSize).observe(previewWrapper.value)
  new ResizeObserver(fixTimelineCanvasSize).observe(timelineCanvasWrapper.value)

  fixPreviewSize()
  fixTimelineCanvasSize()
})
</script>

<style>
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

#timeline {
  min-height: 18rem;

  border-top: solid 1px var(--secondary);

  display: flex;
  flex-direction: column;
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

#control-bar {
  height: 2rem;

  border-bottom: solid 1px var(--secondary);
}

#timeline-canvas-wrapper {
  flex-grow: 1;
}

#timeline-canvas {
  display: block;
  position: absolute;
}
</style>
