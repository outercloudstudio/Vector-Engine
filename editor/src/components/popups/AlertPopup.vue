<template>
  <Transition name="fade">
    <div v-if="display" id="component">
      <div id="popup">
        <p>{{ text }}</p>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { Transition } from 'vue'

defineProps(['text', 'display'])
</script>

<style scoped>
#component {
  position: absolute;

  top: 0;
  left: 0;

  width: 100%;
  height: 100%;

  background: var(--fade);

  z-index: 9999;

  display: flex;
  justify-content: center;
  align-items: center;

  backdrop-filter: blur(2px);

  filter: drop-shadow(0px 4px 4px rgba(0, 0, 0, 0.25));
}

#popup {
  background-color: var(--secondary);

  display: flex;
  flex-direction: column;

  align-items: center;

  width: max-content;
  max-width: 20rem;

  box-sizing: content-box;

  padding: 1rem;

  border-radius: 5px;
}

p {
  display: table;

  margin: 0;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 100ms ease;
}
</style>
