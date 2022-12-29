<template>
  <div id="component">
    <span class="material-symbols-outlined icon-button" @click="router.push({ name: 'Projects' })"> dataset </span>

    <span class="material-symbols-outlined icon-button"> question_mark </span>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import router from '@/router'
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
