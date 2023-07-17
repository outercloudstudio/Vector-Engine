import { get, writable, type Writable } from 'svelte/store'

export const frame: Writable<number> = writable(0)
export const playing: Writable<boolean> = writable(false)

let playStartTime: number
let startFrame: number

function updateFrame() {
	if (!get(playing)) return

	const now = Date.now()

	frame.set(Math.floor(((now - playStartTime) / 1000) * 60) + startFrame)

	requestAnimationFrame(updateFrame)
}

export function play() {
	playStartTime = Date.now()
	startFrame = get(frame)

	playing.set(true)

	updateFrame()
}

export function pause() {
	playing.set(false)
}
