<template>
  <div id="toggle" @click="$emit('update:modelValue', !modelValue)">
    <div id="knob" :class="{ active: modelValue }"></div>
  </div>
</template>

<script setup lang="ts">
defineProps(['modelValue'])
defineEmits(['update:modelValue'])
</script>

<style scoped>
#toggle {
  background: var(--secondary);
  width: 1.75rem;
  height: 0.75rem;

  border-radius: 0.5rem;
}

#knob {
  background: var(--tertiary);
  border-radius: 50%;
  width: 0.75rem;
  height: 0.75rem;
  margin-left: 0;

  transition: background-color 100ms ease-in-out, margin 100ms ease-in-out;
}

#knob.active {
  background: var(--grab);
  margin-left: 1rem;
}
</style>
