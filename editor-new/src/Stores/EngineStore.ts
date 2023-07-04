import { get, writable, type Writable } from 'svelte/store'
import { Engine } from '@vector-engine/core'

export const engine: Writable<Engine | undefined> = writable(undefined)
export const engineProject = writable(undefined)
export const engineData = writable(undefined)
export const frame = writable(0)

export async function makeEngine(newProject: any, newData: any) {
	engineProject.set(newProject)
	engineData.set(newData)

	const newEngine = new Engine(newProject, newData.project)

	await newEngine.load()

	await newEngine.jumpToFrame(get(frame))

	engine.set(newEngine)
}

export const exportInProgress = writable(false)

export async function exportAnimation(name: string) {
	if (get(exportInProgress)) return

	exportInProgress.set(true)

	window.dispatchEvent(
		new CustomEvent('export-start', {
			detail: {
				name: name,
			},
		})
	)

	const engine = new Engine(get(engineProject), get(engineData).project)
	await engine.load()

	const frameDigits = engine.length.toString().length

	for (let frame = 0; frame < engine.length; frame++) {
		const frameName = `${name}/frame_${frame.toString().padStart(frameDigits, '0')}.png`

		await engine.next()

		const render = await engine.render()
		const renderBlob: Blob = await (<any>render).convertToBlob()
		const arrayBuffer = await renderBlob.arrayBuffer()
		const byteArray = new Uint8Array(arrayBuffer, 0, arrayBuffer.byteLength)

		window.dispatchEvent(
			new CustomEvent('export', {
				detail: {
					name: frameName,
					image: byteArray,
				},
			})
		)

		console.log((frame / engine.length) * 100)
	}

	window.dispatchEvent(
		new CustomEvent('export-complete', {
			detail: {
				name,
				length: engine.length,
				frameRate: engine.frameRate,
			},
		})
	)

	exportInProgress.set(false)
}
