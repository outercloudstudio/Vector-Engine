import { get, writable } from 'svelte/store'
import { engine, frame } from './EngineStore'

export const playing = writable(false)
let startedPlayingTime = 0
let startedPlayingFrame = 0

export const speed = writable(1)

export function play() {
	playing.set(true)

	startedPlayingTime = Date.now()
	startedPlayingFrame = get(frame)

	requestAnimationFrame(playUpdate)
}

export async function playUpdate() {
	if (!get(playing)) return

	const engineValue = get(engine)

	const now = Date.now()
	let newFrame =
		Math.floor(((now - startedPlayingTime) / 1000) * get(speed) * engineValue.frameRate) +
		startedPlayingFrame

	if (newFrame > engineValue.length - 1) newFrame = engineValue.length - 1

	await engineValue.jumpToFrame(newFrame)
	frame.set(newFrame)

	if (engineValue.frame >= engineValue.length - 1) {
		startedPlayingTime = Date.now()

		startedPlayingFrame = 0
	}

	requestAnimationFrame(playUpdate)
}

export function pause() {
	playing.set(false)
}

export async function restart() {
	pause()

	await get(engine).jumpToFrame(0)
	frame.set(0)
}

export async function back() {
	pause()

	const newFrame = get(frame) - 1

	if (newFrame < 0) return

	await get(engine).jumpToFrame(newFrame)
	frame.set(newFrame)
}

export async function next() {
	pause()

	const newFrame = get(frame) + 1

	if (newFrame >= get(engine).length) return

	await get(engine).jumpToFrame(newFrame)
	frame.set(newFrame)
}
