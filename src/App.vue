<script setup lang="ts">
import { invoke } from '@tauri-apps/api'
import { onMounted, ref } from 'vue'

let frames: string[] = []

let imageSrc = ref('')
let frameRate = ref(0)

let animationStart = Date.now()

let lastFrameTime = Date.now()
let lastFrame = -1

const length = ref(100)

async function preview() {
	const now = Date.now()
	let frame = Math.floor(((now - animationStart) / 1000) * 60)

	if (frame >= length.value) {
		frame = 0

		animationStart = now
	}

	if (lastFrame !== frame) {
		frameRate.value = (1 / (now - lastFrameTime)) * 1000
		lastFrameTime = now

		if (frames[frame] !== undefined) {
			imageSrc.value = frames[frame]
		} else {
			const arrayBuffer = await (
				await fetch(`https://preview.localhost/?frame=${frame}`)
			).arrayBuffer()
			const blob = new Blob([arrayBuffer], { type: 'image/bmp' })
			const src = window.URL.createObjectURL(blob)

			setTimeout(() => {
				URL.revokeObjectURL(src)
			}, 10)

			// frames[frame] = src    Disabled untill better support on rust for reload

			imageSrc.value = src
		}
	}

	lastFrame = frame

	requestAnimationFrame(preview)
}

onMounted(() => {
	preview()
})
</script>

<template>
	<p>{{ Math.floor(frameRate) }}</p>
	<img class="preview" :src="imageSrc" />
	<button @click="invoke('render', { length })">Render</button>
	<input type="number" v-model="length" />
</template>

<style scoped>
.preview {
	width: calc(1920px / 4);
	height: calc(1080px / 4);
	margin-right: 1rem;
}
</style>
