import { get, writable, type Writable } from 'svelte/store'

export type Clip = {
	id: string
	assetId: string
	frame: number
	length: number
}

export const layers: Writable<{
	[key: number]: Clip[]
}> = writable({})

export function nextValidFrame(frame: number, length: number, layer: number) {
	let nextOpenSpace = Math.max(0, frame)

	const layersReference = get(layers)

	if (layersReference[layer] === undefined) return nextOpenSpace

	for (let clipIndex = 0; clipIndex < layersReference[layer].length; clipIndex++) {
		const startIntersects =
			layersReference[layer][clipIndex].frame >= nextOpenSpace &&
			layersReference[layer][clipIndex].frame < nextOpenSpace + length

		const endIntersects =
			layersReference[layer][clipIndex].frame + layersReference[layer][clipIndex].length >=
				nextOpenSpace &&
			layersReference[layer][clipIndex].frame + layersReference[layer][clipIndex].length <
				nextOpenSpace + length

		const otherStartIntersects =
			nextOpenSpace >= layersReference[layer][clipIndex].frame &&
			nextOpenSpace <
				layersReference[layer][clipIndex].frame + layersReference[layer][clipIndex].length

		const otherEndIntersects =
			nextOpenSpace + length >= layersReference[layer][clipIndex].frame &&
			nextOpenSpace + length <
				layersReference[layer][clipIndex].frame + layersReference[layer][clipIndex].length

		if (startIntersects || endIntersects || otherStartIntersects || otherEndIntersects)
			nextOpenSpace =
				layersReference[layer][clipIndex].frame + layersReference[layer][clipIndex].length
	}

	return nextOpenSpace
}

export function addClip(
	assetId: string,
	frame: number,
	length: number,
	layer: number,
	id?: string
) {
	const layersReference = get(layers)

	if (layersReference[layer] === undefined) layersReference[layer] = []

	let insertFrame = layersReference[layer].findIndex(clip => clip.frame >= frame)

	if (insertFrame === -1) insertFrame = layersReference[layer].length

	layersReference[layer].splice(insertFrame, 0, {
		id: id || self.crypto.randomUUID(),
		assetId,
		frame: nextValidFrame(frame, length, layer),
		length,
	})

	layers.set(layersReference)
}

export function removeClip(id: string): Clip {
	const layersReference = get(layers)

	for (const layer of Object.keys(layersReference)) {
		const clipIndex = layersReference[layer].findIndex(clip => clip.id === id)

		if (clipIndex !== -1) {
			layersReference[layer].splice(clipIndex, 1)

			layers.set(layersReference)

			return layersReference[layer][clipIndex]
		}
	}
}

export function findClipLocation(id: string): { layer: number; index: number } | null {
	const layersReference = get(layers)

	for (const layer of Object.keys(layersReference).map(layer => parseInt(layer))) {
		for (let clipIndex = 0; clipIndex < layersReference[layer].length; clipIndex++) {
			if (layersReference[layer][clipIndex].id === id) return { layer, index: clipIndex }
		}
	}

	return null
}

export function clipsAtFrame(frame: number): Clip[] {
	const layersReference = get(layers)

	let clips: Clip[] = []

	for (const layer of Object.keys(layersReference).map(layer => parseInt(layer))) {
		for (let clipIndex = 0; clipIndex < layersReference[layer].length; clipIndex++) {
			const clip = layersReference[layer][clipIndex]
			if (frame >= clip.frame && frame < clip.frame + clip.length) clips.push(clip)
		}
	}

	return clips
}
