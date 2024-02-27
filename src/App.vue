<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api'

let imageSrc = ref('')
let imageSrc2 = ref('')

onMounted(async () => {
	invoke('render')

	// const [data, data2] = (await invoke('preview')) as number[][]

	// const arrayBuffer = new Uint8Array(data)
	// const blob = new Blob([arrayBuffer], { type: 'image/png' })
	// const src = window.URL.createObjectURL(blob)

	// imageSrc.value = src

	// const arrayBuffer2 = new Uint8Array(data2)
	// const blob2 = new Blob([arrayBuffer2], { type: 'image/png' })
	// const src2 = window.URL.createObjectURL(blob2)

	// imageSrc2.value = src2
})
</script>

<template>
	<img class="preview" :src="imageSrc" />
	<img class="preview" :src="imageSrc2" />
</template>

<style scoped>
.preview {
	width: 248px;
	height: 248px;
	margin-right: 1rem;
}
</style>
