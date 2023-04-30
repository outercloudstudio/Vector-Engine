<script lang="ts">
	import { get } from 'svelte/store'
	import { back, next, pause, play, playing, restart } from '../Stores/PlayStore'
	import { engine, frame } from '../Stores/EngineStore'

	function playOrPause() {
		if (get(playing)) {
			pause()
		} else {
			play()
		}
	}
</script>

<main>
	<p>{$frame}</p>

	<div>
		<span class="material-symbols-outlined icon-button" on:mouseup={restart}> fast_rewind </span>

		<span class="material-symbols-outlined icon-button" on:mouseup={back}> skip_previous </span>

		<span class="material-symbols-outlined icon-button" on:mouseup={playOrPause}>
			{$playing ? 'pause' : 'play_arrow'}
		</span>

		<span class="material-symbols-outlined icon-button" on:mouseup={next}> skip_next </span>
	</div>

	<p>{$engine ? $engine.length : 60}</p>
</main>

<style>
	main {
		height: 2rem;

		display: flex;
		justify-content: space-between;
		align-items: center;
		padding-left: 1rem;
		padding-right: 1rem;
	}

	div {
		display: flex;

		align-items: center;
	}

	p {
		display: inline;

		margin: 0;
		padding: 0;

		color: var(--text);
		font-size: small;
	}
</style>
