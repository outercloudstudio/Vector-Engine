<script lang="ts">
	import { onDestroy, onMount } from 'svelte'
	import { scenes } from '../stores/projectStore'

	const tabs = ['Clips', 'Export', 'Inspect', 'Debug']
	let activeTab = tabs[0]

	let heldClip: string | null = null
	let heldX: number = 0
	let heldY: number = 0

	function holdClip(event: MouseEvent, scene: string) {
		event.preventDefault()

		heldClip = scene

		heldX = event.clientX
		heldY = event.clientY
	}

	onMount(() => {
		window.addEventListener('mousemove', moveClip)
		window.addEventListener('mouseup', dropClip)
	})

	onDestroy(() => {
		window.removeEventListener('mousemove', moveClip)
		window.removeEventListener('mouseup', dropClip)
	})

	function moveClip(event: MouseEvent) {
		if (heldClip === null) return

		heldX = event.clientX
		heldY = event.clientY
	}

	function dropClip(event: MouseEvent) {
		if (heldClip === null) return

		heldClip = null

		console.log('Dropped')
	}
</script>

<main>
	<div class="tabs">
		{#each tabs as tab}
			<button on:click={() => (activeTab = tab)} class={activeTab === tab ? 'active' : ''}
				>{tab}</button
			>
		{/each}
	</div>

	{#if activeTab === 'Clips'}
		<div class="content">
			{#each Object.keys($scenes) as scene}
				<div on:mousedown={event => holdClip(event, scene)} class="clip">
					<p>{scene}</p>
				</div>
			{/each}
		</div>

		{#if heldClip !== null}
			<div id="held-clip" class="clip" style="left: {heldX}px; top: {heldY}px;">
				<p>{heldClip}</p>
			</div>
		{/if}
	{/if}
</main>

<style>
	main {
		background-color: var(--main);

		grid-column-start: 1;
		grid-column-end: 2;
		grid-row-start: 1;
		grid-row-end: 2;

		border-right: solid 1px var(--secondary);
	}

	.tabs {
		display: flex;
		justify-content: space-between;

		border-bottom: solid 1px var(--secondary);
	}

	.tabs > button {
		flex-grow: 1;

		appearance: none;

		border: none;

		background: var(--main);

		height: 30px;
	}

	.tabs > button:hover {
		background: var(--secondary);
	}

	.tabs > button.active {
		background: var(--secondary);
	}

	.content {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-around;
		align-items: start;
		align-content: start;

		padding: 10px;
		gap: 10px;
	}

	.clip {
		width: 150px;
		height: 75px;

		background: var(--secondary);

		border-radius: 5px;

		display: flex;
		justify-content: center;
		align-items: center;
	}

	.clip > p {
		user-select: none;
	}

	#held-clip {
		position: absolute;

		transform: translate(-50%, -50%);

		filter: drop-shadow(0px 0px 8px #0000004a);
	}
</style>
