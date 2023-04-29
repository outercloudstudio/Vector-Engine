<script lang="ts">
	import { engine, engineData, engineProject, frame, makeEngine } from './stores.js'
	import Preview from './Preview.svelte'
	import { onMount } from 'svelte'
	import { get } from 'svelte/store'

	onMount(async () => {
		window.addEventListener('project', async project => {
			await makeEngine((<CustomEvent>project).detail.project, (<CustomEvent>project).detail.data)

			setInterval(() => {
				get(engine).next()
				frame.update(value => value + 1)
			}, 1000 / 60)
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
</main>

<style>
	main {
		display: flex;
		flex-direction: column;

		width: 100%;
		height: 100%;
	}
</style>
