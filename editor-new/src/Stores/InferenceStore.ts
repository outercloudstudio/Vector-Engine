import { get, writable } from 'svelte/store'
import { engine } from './EngineStore'

export const audioInference = writable([])

engine.subscribe(() => {
	inferenceAudio()
})

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
