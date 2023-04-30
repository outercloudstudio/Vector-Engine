<script lang="ts">
	import Preview from './Components/Preview.svelte'
	import Timeline from './Components/Timeline.svelte'

	import { engine, engineData, engineProject, frame, makeEngine } from './Stores/EngineStore.js'
	import { onMount } from 'svelte'
	import { get } from 'svelte/store'
	import { play } from './Stores/PlayStore'
	import ControlBar from './Components/ControlBar.svelte'

	onMount(async () => {
		window.addEventListener('project', async project => {
			await makeEngine((<CustomEvent>project).detail.project, (<CustomEvent>project).detail.data)

			play()
		})

		window.addEventListener('project-update', async project => {
			await makeEngine((<CustomEvent>project).detail, get(engineData))
		})

		window.addEventListener('data-update', async data => {
			await makeEngine(get(engineProject), (<CustomEvent>data).detail)
		})
	})
</script>

<main>
	<Preview />
	<ControlBar />
	<Timeline />
</main>

<style>
	main {
		display: flex;
		flex-direction: column;

		width: 100%;
		height: 100%;
	}
</style>
