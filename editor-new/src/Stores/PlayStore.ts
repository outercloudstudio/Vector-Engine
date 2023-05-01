import { get, writable } from 'svelte/store'
import { engine, frame } from './EngineStore'

export const playing = writable(false)
let startedPlayingTime = 0
let startedPlayingFrame = 0

export const playFrameRate = writable(0)
let lastFrameTime = 0
let lastFrameRateUpdateTime = 0

export const speed = writable(1)

const audioContext: AudioContext = new AudioContext({
	latencyHint: 'interactive',
})
const audioGain: GainNode = audioContext.createGain()
audioGain.connect(audioContext.destination)
const audioDestination: AudioNode = audioGain

let audioTrackBufferSource: AudioBufferSourceNode | undefined = undefined

async function startAudioPlayback(time: number) {
	const audioBuffer = get(engine).audioTrack

	if (!audioBuffer) return

	audioTrackBufferSource = audioContext.createBufferSource()
	audioTrackBufferSource.buffer = audioBuffer
	audioTrackBufferSource.connect(audioDestination)

	audioContext.resume()

	audioTrackBufferSource.start(0, time)
}

function stopAudioPlayback() {
	if (!audioTrackBufferSource) return

	audioTrackBufferSource.stop()
}

export async function play() {
	playing.set(true)

	const currentFrame = get(frame)
	const currentEngine = get(engine)

	startedPlayingTime = Date.now()
	startedPlayingFrame = currentFrame

	lastFrameTime = startedPlayingTime
	lastFrameRateUpdateTime = startedPlayingTime

	await startAudioPlayback(currentFrame / currentEngine.frameRate)

	requestAnimationFrame(playUpdate)
}

export async function playUpdate() {
	if (!get(playing)) return

	const engineValue = get(engine)

	const now = Date.now()
	let newFrame =
		Math.floor(((now - startedPlayingTime) / 1000) * get(speed) * engineValue.frameRate) +
		startedPlayingFrame

	if (now - lastFrameRateUpdateTime > 300) {
		playFrameRate.set(Math.floor((1 / (now - lastFrameTime)) * 1000))

		lastFrameRateUpdateTime = now
	}

	lastFrameTime = now

	if (newFrame > engineValue.length - 1) newFrame = engineValue.length - 1

	await engineValue.jumpToFrame(newFrame)
	frame.set(newFrame)

	if (engineValue.frame >= engineValue.length - 1) {
		startedPlayingTime = Date.now()

		startedPlayingFrame = 0

		stopAudioPlayback()
		startAudioPlayback(0)
	}

	requestAnimationFrame(playUpdate)
}

export function pause() {
	playing.set(false)

	stopAudioPlayback()
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
