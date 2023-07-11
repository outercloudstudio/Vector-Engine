<script lang="ts">
	import Timeline from './components/Timeline.svelte'
	import Sidebar from './components/Sidebar.svelte'
	import Preview from './components/Preview.svelte'

	import { onMount } from 'svelte'

	let canvas: HTMLCanvasElement

	onMount(() => {})
</script>

<main>
	<!-- <canvas bind:this={canvas} width="640" height="480" /> -->
	<div class="grid">
		<Sidebar />
		<Preview />
		<Timeline />
	</div>
</main>

<style>
	.grid {
		display: grid;
		grid-template-rows: 75% 25%;
		grid-template-columns: 20% 80%;

		width: 100vw;
		height: 100vh;
	}
</style>
