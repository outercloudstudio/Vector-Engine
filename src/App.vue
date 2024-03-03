<script setup lang="ts">
import { onMounted, ref } from 'vue'

let imageSrc = ref('')
let frameRate = ref(0)

let frame = 0
let lastFrameTime = Date.now()

async function preview() {
	let currentFrameTime = Date.now()
	frameRate.value = (1 / (currentFrameTime - lastFrameTime)) * 1000
	lastFrameTime = currentFrameTime

	// await fetch('https://preview.localhost/')

	const arrayBuffer = await (await fetch('https://preview.localhost/')).arrayBuffer()
	const blob = new Blob([arrayBuffer], { type: 'image/png' })
	const src = window.URL.createObjectURL(blob)

	imageSrc.value = src

	frame++

	if (frame === 60) frame = 0

	requestAnimationFrame(preview)
}

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
	width: calc(1920px / 4);
	height: calc(1080px / 4);
	margin-right: 1rem;
}
</style>
