<script lang="ts">
	import { dropped, droppedOn, held } from '../stores/heldStore'

	let componentBody: HTMLElement

	$: if ($dropped !== null && droppedOn(componentBody)) {
		console.log('Dropped!')
	}
</script>

<main bind:this={componentBody} />

<style>
	main {
		background-color: var(--main);

		grid-column-start: 1;
		grid-column-end: 3;
		grid-row-start: 2;
		grid-row-end: 3;

		border-top: solid 1px var(--secondary);
	}
</style>
