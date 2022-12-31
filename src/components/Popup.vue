<template>
  <div id="component">
    <div id="popup">
      <p>{{ text }}</p>

      <button @click="$emit('confirmed')">{{ buttonText }}</button>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps(['text', 'buttonText'])
defineEmits(['confirmed'])
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
  margin-bottom: 1rem;
}

button {
  background-color: var(--grab);
  border-radius: 5px;

  padding: 0.25rem 0.5rem;

  transition: transform 200ms ease-in-out;
}

button:hover {
  transform: scale(1.05);
}
</style>
