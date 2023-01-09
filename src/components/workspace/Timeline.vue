<template>
  <div id="component">
    <div id="control-bar">
      <div class="control-bar-group">
        <input value="0:00" id="time-input" />
        <input value="[0]" id="frame-input" />
      </div>

      <div class="control-bar-group">
        <span class="material-symbols-outlined icon-button"> volume_up </span>

        <span class="material-symbols-outlined icon-button"> fast_rewind </span>
      </div>

      <div class="control-bar-group">
        <span class="material-symbols-outlined icon-button">
          skip_previous
        </span>

        <span class="material-symbols-outlined icon-button"> play_arrow </span>

        <span class="material-symbols-outlined icon-button"> skip_next </span>
      </div>

      <div class="control-bar-group">
        <span class="material-symbols-outlined icon-button"> redo </span>
      </div>

      <div class="control-bar-group">
        <p id="frame-length">[60]</p>
        <p id="length">0:01</p>
      </div>
    </div>
    <div ref="canvasWrapper" id="canvas-wrapper">
      <canvas ref="canvas" id="canvas"></canvas>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, Ref, ref } from 'vue'

const canvas: Ref<null | HTMLCanvasElement> = ref(null)
const canvasWrapper: Ref<null | HTMLDivElement> = ref(null)

function fixCanvasSize() {
  if (!canvasWrapper.value) return
  if (!canvas.value) return

  canvas.value.width = canvasWrapper.value.offsetWidth
  canvas.value.height = canvasWrapper.value.offsetHeight
}

onMounted(() => {
  if (!canvasWrapper.value) return

  new ResizeObserver(fixCanvasSize).observe(canvasWrapper.value)

  fixCanvasSize()
})
</script>

<style scoped>
#component {
  min-height: 18rem;

  border-top: solid 1px var(--secondary);

  display: flex;
  flex-direction: column;
}

#control-bar {
  height: 2rem;

  border-bottom: solid 1px var(--secondary);

  display: flex;
  justify-content: space-between;
  align-items: center;

  box-sizing: border-box;

  padding-left: 0.5rem;
  padding-right: 0.5rem;
}

#canvas-wrapper {
  flex-grow: 1;
}

#canvas {
  display: block;
  position: absolute;
}

.control-bar-group {
  display: flex;
  gap: 0.5rem;
}

#time-input {
  font-size: small;
}

#frame-input {
  font-size: 0.7rem;

  color: var(--alternate-text);

  margin-top: auto;
  margin-bottom: auto;
}

#time-input,
#frame-input {
  width: 3rem;
  background: none;
  padding: 0;
}

#frame-length {
  font-size: 0.7rem;

  color: var(--alternate-text);

  margin: 0;

  width: 3rem;

  text-align: right;

  margin-top: auto;
  margin-bottom: auto;
}

#length {
  font-size: small;

  margin: 0;

  width: 3rem;

  text-align: right;
}
</style>
