<template>
  <div ref="wrapper" id="wrapper">
    <canvas
      ref="preview"
      id="preview"
      @mousedown="mouseDown"
      @mouseup="() => (mouse = false)"
      @mouseleave="() => (mouse = false)"
      @mousemove="mouseMove"
      @wheel="scroll"
    ></canvas>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, Ref, watch, render } from 'vue'
import { useWorkspaceStore } from '@/stores/WorkspaceStore'

const WorkspaceStore = useWorkspaceStore()

const props = defineProps(['render'])

const offsetX = ref(0)
const offsetY = ref(0)

const preview: Ref<null | HTMLCanvasElement> = ref(null)
const wrapper: Ref<null | HTMLDivElement> = ref(null)

let mouse = false
let mouseDownX = 0
let mouseDownY = 0

function mouseDown(event: MouseEvent) {
  mouse = true

  mouseDownX = event.clientX
  mouseDownY = event.clientY
}

function mouseMove(event: MouseEvent) {
  if (!mouse) return
  if (!preview.value) return

  offsetX.value += event.clientX - mouseDownX
  offsetY.value += event.clientY - mouseDownY

  mouseDownX = event.clientX
  mouseDownY = event.clientY
}

let zoom = ref(1)

function scroll(event: any) {
  const scrollY = event.deltaY / 30000

  zoom.value = Math.pow(Math.pow(zoom.value, 1 / 10) - scrollY, 10)

  zoom.value = Math.max(zoom.value, 0.0001)
}

async function renderCanvas() {
  if (!wrapper.value) return
  if (!preview.value) return

  const render = await WorkspaceStore.render()

  if (!render) return

  preview.value
    .getContext('2d')
    ?.clearRect(0, 0, preview.value.width, preview.value.height)

  const ratioWidth = preview.value.width
  const ratioHeight = preview.value.height * (16 / 9)

  const bestWidth = Math.min(ratioWidth, ratioHeight)

  const bestHeight = bestWidth * (9 / 16)
  const scaledWidth = bestWidth * zoom.value
  const scaledHeight = bestHeight * zoom.value

  preview.value
    .getContext('2d')
    ?.drawImage(
      render,
      wrapper.value.offsetWidth / 2 + offsetX.value - scaledWidth / 2,
      wrapper.value.offsetHeight / 2 + offsetY.value - scaledHeight / 2,
      scaledWidth,
      scaledHeight
    )
}

watch(
  () => props.render,
  render => {
    if (!render) return

    renderCanvas()
  }
)

watch(offsetX, async () => {
  if (!render) return

  renderCanvas()
})

watch(offsetY, async () => {
  if (!render) return

  renderCanvas()
})

watch(zoom, async () => {
  if (!render) return

  renderCanvas()
})

watch(
  () => WorkspaceStore.frame,
  async () => {
    if (!render) return

    renderCanvas()
  }
)

function fixPreviewSize() {
  if (!wrapper.value) return
  if (!preview.value) return

  preview.value.width = wrapper.value.offsetWidth
  preview.value.height = wrapper.value.offsetHeight

  renderCanvas()
}

onMounted(() => {
  if (!wrapper.value) return

  new ResizeObserver(fixPreviewSize).observe(wrapper.value)

  fixPreviewSize()
})
</script>

<style scoped>
#wrapper {
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
</style>
