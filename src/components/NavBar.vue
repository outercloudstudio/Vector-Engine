<template>
  <div id="component">
    <span class="material-symbols-outlined icon-button" @click="navigate">
      {{ leftIcon }}
    </span>

    <span class="material-symbols-outlined icon-button"> question_mark </span>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
const router = useRouter()

const props = defineProps(['leftIcon', 'leftLink'])

function navigate() {
  router.push({ name: props.leftLink })
}
</script>

<style scoped>
#component {
  position: absolute;

  width: 100%;
  height: 2rem;

  background-color: var(--main);

  border-bottom: 1px solid var(--secondary);

  z-index: 1;

  display: flex;
  justify-content: space-between;
  align-items: center;

  box-sizing: border-box;

  padding-right: 0.5rem;
  padding-left: 0.5rem;
}
</style>
