<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api'
import { listen } from '@tauri-apps/api/event'

let imageSrc = ref('')
let frameRate = ref(0)

let frame = 0
let lastFrameTime = Date.now()

async function preview() {
	let currentFrameTime = Date.now()
	frameRate.value = (1 / (currentFrameTime - lastFrameTime)) * 1000
	lastFrameTime = currentFrameTime

	const data = (await invoke('preview', { frame })) as number[]

	const arrayBuffer = new Uint8Array(data)
	const blob = new Blob([arrayBuffer], { type: 'image/png' })
	const src = window.URL.createObjectURL(blob)

	imageSrc.value = src

	frame++

	if (frame === 60) frame = 0

	requestAnimationFrame(preview)
}

listen('render', event => {
	const arrayBuffer = new Uint8Array(event.payload as number[])
	const blob = new Blob([arrayBuffer], { type: 'image/png' })
	const src = window.URL.createObjectURL(blob)

	imageSrc.value = src
})

onMounted(() => {
	preview()
})
</script>

<template>
	<p>{{ frameRate }}</p>
	<img class="preview" :src="imageSrc" />

	<button @click="preview">Preview</button>
</template>

<style scoped>
.preview {
	width: calc(1920px / 3);
	height: calc(1080px / 3);
	margin-right: 1rem;
}
</style>
