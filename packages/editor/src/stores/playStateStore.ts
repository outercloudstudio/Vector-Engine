import { get, writable, type Writable } from 'svelte/store'

export const frame: Writable<number> = writable(0)
export const playing: Writable<boolean> = writable(false)

let playStartTime: number

function updateFrame() {
	if (!get(playing)) return

	const now = Date.now()

	frame.set(Math.floor(((now - playStartTime) / 1000) * 60))

	requestAnimationFrame(updateFrame)
}

export function play() {
	playStartTime = Date.now()

	playing.set(true)

	updateFrame()
}

export function stop() {
	playing.set(false)
}
