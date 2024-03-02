<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api'

let imageSrc = ref('')

let frame = 0

async function preview() {
	const data = (await invoke('preview', { frame })) as number[]

	const arrayBuffer = new Uint8Array(data)
	const blob = new Blob([arrayBuffer], { type: 'image/png' })
	const src = window.URL.createObjectURL(blob)

	imageSrc.value = src

	frame++

	if (frame === 60) frame = 0

	setTimeout(preview, 1000 / 60)
}

onMounted(() => {
	preview()
})
</script>

<template>
	<img class="preview" :src="imageSrc" />

	<button @click="preview">Preview</button>
</template>

<style scoped>
.preview {
	width: 248px;
	height: 248px;
	margin-right: 1rem;
}
</style>
