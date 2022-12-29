<template>
  <h1>Vector Engine</h1>

  <button @click="router.push({ name: 'Projects' })">Go!</button>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import router from '@/router'
</script>

<style scoped></style>
