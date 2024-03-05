<script setup lang="ts">
import { onMounted, ref } from 'vue'

let imageSrc = ref('')
let frameRate = ref(0)

let animationStart = Date.now()

let lastFrameTime = Date.now()

async function preview() {
	const now = Date.now()
	let frame = Math.floor(((now - animationStart) / 1000) * 60)

	if (frame >= 120) {
		frame = 0

		animationStart = now
	}

	frameRate.value = (1 / (now - lastFrameTime)) * 1000
	lastFrameTime = now

	const arrayBuffer = await (await fetch(`https://preview.localhost/?frame=${frame}`)).arrayBuffer()
	const blob = new Blob([arrayBuffer], { type: 'image/png' })
	const src = window.URL.createObjectURL(blob)

	imageSrc.value = src

	requestAnimationFrame(preview)
}

onMounted(() => {
	preview()
})
</script>

<template>
	<p>{{ frameRate }}</p>
	<img class="preview" :src="imageSrc" />
</template>

<style scoped>
.preview {
	width: calc(1920px / 4);
	height: calc(1080px / 4);
	margin-right: 1rem;
}
</style>
