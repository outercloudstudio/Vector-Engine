<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api'

let imageSrc = ref('')

onMounted(async () => {
	const data = (await invoke('preview')) as number[]

	const arrayBuffer = new Uint8Array(data)
	const blob = new Blob([arrayBuffer], { type: 'image/png' })
	const src = window.URL.createObjectURL(blob)

	imageSrc.value = src
})
</script>

<template>
	<img id="preview" :src="imageSrc" />
</template>

<style scoped>
#preview {
	width: 512px;
	height: 512px;
}
</style>
