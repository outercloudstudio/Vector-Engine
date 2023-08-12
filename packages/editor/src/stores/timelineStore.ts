import type { Asset } from '@vector-engine/core'
import { get, writable, type Writable } from 'svelte/store'
import { assets, project, saveProject } from './projectStore'

export type Clip = {
	id: string
	assetId: string
	asset: Asset
	frame: number
	firstClipFrame: number
	length: number
}

export const timeline: Writable<{
	[key: number]: Clip[]
}> = writable({})

export function nextValidFrame(frame: number, length: number, layer: number) {
	let nextOpenSpace = Math.max(0, frame)

	const timelineReference = get(timeline)

	if (timelineReference[layer] === undefined) return nextOpenSpace

	for (let clipIndex = 0; clipIndex < timelineReference[layer].length; clipIndex++) {
		const startIntersects =
			timelineReference[layer][clipIndex].frame >= nextOpenSpace &&
			timelineReference[layer][clipIndex].frame < nextOpenSpace + length

		const endIntersects =
			timelineReference[layer][clipIndex].frame + timelineReference[layer][clipIndex].length >=
				nextOpenSpace &&
			timelineReference[layer][clipIndex].frame + timelineReference[layer][clipIndex].length <
				nextOpenSpace + length

		const otherStartIntersects =
			nextOpenSpace >= timelineReference[layer][clipIndex].frame &&
			nextOpenSpace <
				timelineReference[layer][clipIndex].frame + timelineReference[layer][clipIndex].length

		const otherEndIntersects =
			nextOpenSpace + length >= timelineReference[layer][clipIndex].frame &&
			nextOpenSpace + length <
				timelineReference[layer][clipIndex].frame + timelineReference[layer][clipIndex].length

		if (startIntersects || endIntersects || otherStartIntersects || otherEndIntersects)
			nextOpenSpace =
				timelineReference[layer][clipIndex].frame + timelineReference[layer][clipIndex].length
	}

	return nextOpenSpace
}

// We can't just save this timeline to the project timeline since this timelien has asset instances
export function saveTimeline() {
	const timelineReference = get(timeline)

	const projectReference = get(project)

	projectReference.timeline = {}

	for (const layer of Object.keys(timelineReference).map(layer => parseInt(layer))) {
		projectReference.timeline[layer] = timelineReference[layer].map(clip => {
			return {
				id: clip.id,
				assetId: clip.assetId,
				frame: clip.frame,
				firstClipFrame: clip.firstClipFrame,
				length: clip.length,
			}
		})
	}

	saveProject()
}

export function addClip(
	assetId: string,
	asset: Asset,
	frame: number,
	firstClipFrame: number,
	length: number,
	layer: number,
	id?: string,
	save: boolean = true
) {
	const timelineReference = get(timeline)

	if (timelineReference[layer] === undefined) timelineReference[layer] = []

	let insertFrame = timelineReference[layer].findIndex(clip => clip.frame >= frame)

	if (insertFrame === -1) insertFrame = timelineReference[layer].length

	timelineReference[layer].splice(insertFrame, 0, {
		id: id || self.crypto.randomUUID(),
		assetId,
		asset,
		frame: nextValidFrame(frame, length, layer),
		firstClipFrame,
		length,
	})

	timeline.set(timelineReference)

	if (save) saveTimeline()
}

export function removeClip(id: string): Clip {
	const timelineReference = get(timeline)

	for (const layer of Object.keys(timelineReference)) {
		const clipIndex = timelineReference[layer].findIndex(clip => clip.id === id)

		if (clipIndex !== -1) {
			timelineReference[layer].splice(clipIndex, 1)

			timeline.set(timelineReference)

			saveTimeline()

			return timelineReference[layer][clipIndex]
		}
	}
}

export function findClipLocation(id: string): { layer: number; index: number } | null {
	const timelineReference = get(timeline)

	for (const layer of Object.keys(timelineReference).map(layer => parseInt(layer))) {
		for (let clipIndex = 0; clipIndex < timelineReference[layer].length; clipIndex++) {
			if (timelineReference[layer][clipIndex].id === id) return { layer, index: clipIndex }
		}
	}

	return null
}

export function clipsAtFrame(frame: number): Clip[] {
	const timelineReference = get(timeline)

	let clips: Clip[] = []

	for (const layer of Object.keys(timelineReference).map(layer => parseInt(layer))) {
		for (let clipIndex = 0; clipIndex < timelineReference[layer].length; clipIndex++) {
			const clip = timelineReference[layer][clipIndex]
			if (frame >= clip.frame && frame < clip.frame + clip.length) clips.push(clip)
		}
	}

	return clips
}

assets.subscribe(assets => {
	const timelineReference = get(timeline)

	for (const layer of Object.keys(timelineReference).map(layer => parseInt(layer))) {
		for (const clip of timelineReference[layer]) {
			clip.asset = assets[clip.assetId]()
		}
	}
})

// We can't just set timeline to the project because the project timeline doesn't contain asset instances
project.subscribe(projectReference => {
	timeline.set({})

	const assetsReference = get(assets)

	for (const layer of Object.keys(projectReference.timeline).map(layer => parseInt(layer))) {
		for (const clip of projectReference.timeline[layer]) {
			addClip(
				clip.assetId,
				assetsReference[clip.assetId](),
				clip.frame,
				clip.firstClipFrame,
				clip.length,
				layer,
				clip.id,
				false
			)
		}
	}
})
