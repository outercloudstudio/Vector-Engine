<template>
  <div id="component" :class="{ selected }">
    <div id="preview"></div>

    <p id="name">{{ title }}</p>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import router from '@/router'

defineProps(['title', 'selected'])
</script>

<style scoped>
#component {
  margin: 0.5rem;
}

#preview {
  width: 9rem;
  height: 6rem;

  background-color: var(--secondary);

  border-radius: 5px;

  box-sizing: border-box;

  border: solid 0px rgba(0, 0, 0, 0);

  transition: border 100ms ease-in-out;
}

#component:hover #preview,
#component.selected #preview {
  border: solid 2px var(--grab);
}

#name {
  margin: 0;
  margin-top: 0.5rem;

  font-size: small;
  text-align: center;

  transition: color 100ms ease-in-out;

  color: var(--alternate-text);
}

#component:hover #name,
#component.selected #name {
  color: var(--grab);
}
</style>
