import { get, writable, type Writable } from 'svelte/store'
import { engine, engineData, engineProject } from './EngineStore'
import { Engine } from '@vector-engine/core'

export const audioInference = writable([])

export function inferenceAudio() {
	audioInference.set([])

	const currentEngine = get(engine)

	if (currentEngine === undefined) return
	if (currentEngine.audioTrack === undefined) return

	const channels = []

	for (
		let channelIndex = 0;
		channelIndex < currentEngine.audioTrack.numberOfChannels;
		channelIndex++
	) {
		channels.push(currentEngine.audioTrack.getChannelData(channelIndex))
	}

	let volumePerFrame: number[] = []

	const samplesPerFrame = (currentEngine.audioTrack.sampleRate || 0) / currentEngine.frameRate

	let peak = 0

	for (let frame = 0; frame < currentEngine.length; frame++) {
		const startingSample = Math.floor(samplesPerFrame * frame)
		let frameValue = 0

		for (const channel of channels) {
			for (
				let sample = startingSample;
				sample < Math.min(startingSample + Math.floor(samplesPerFrame), channel.length);
				sample++
			) {
				frameValue = Math.max(Math.abs(channel[sample]), frameValue)
			}
		}

		const squaredValue = Math.pow(frameValue, 3)

		volumePerFrame.push(squaredValue)

		peak = Math.max(squaredValue, peak)
	}

	for (let frame = 0; frame < currentEngine.length; frame++) {
		volumePerFrame[frame] = Math.min(volumePerFrame[frame] / Math.max(peak, 0.0001), 1)
	}

	audioInference.set(volumePerFrame)
}

export const markerInference: Writable<
	{
		name: string
		frame: number
		length: number
	}[]
> = writable([])

export const sceneInference: Writable<
	{
		name: string
		frame: number
		length: number
	}[]
> = writable([])

export async function inferenceSimulated() {
	const markers = []
	const scenes = []

	const inferenceEngine = new Engine(get(engineProject), get(engineData).project)
	inferenceEngine.referencedMarker = name => {
		const currentEngineData = get(engineData)

		if (currentEngineData.project.markers[name] === undefined) {
			currentEngineData.project.markers[name] = 0.1

			window.dispatchEvent(new CustomEvent('update-data', { detail: currentEngineData }))
		}

		markers.push({
			name,
			frame: inferenceEngine.frame,
			length: Math.floor(currentEngineData.project.markers[name] * inferenceEngine.frameRate),
		})
	}

	await inferenceEngine.load()

	let lastSceneStart = 0
	let lastSceneName = inferenceEngine.currentScene.name

	for (let frame = 0; frame < inferenceEngine.length; frame++) {
		await inferenceEngine.next()

		if (inferenceEngine.currentScene.name !== lastSceneName) {
			scenes.push({
				name: inferenceEngine.currentScene.name,
				frame: lastSceneStart,
				length: frame - lastSceneStart,
			})

			lastSceneStart = frame
			lastSceneName = inferenceEngine.currentScene.name
		}
	}

	scenes.push({
		name: inferenceEngine.currentScene.name,
		frame: lastSceneStart,
		length: inferenceEngine.length - lastSceneStart - 1,
	})

	markerInference.set(markers)
	sceneInference.set(scenes)
}

engine.subscribe(value => {
	if (value === undefined) return

	inferenceAudio()
	inferenceSimulated()
})
