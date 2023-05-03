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
