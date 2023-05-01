import { get, writable, type Writable } from 'svelte/store'
import { Engine } from '@vector-engine/core'

export const engine: Writable<Engine | undefined> = writable(undefined)
export const engineProject = writable(undefined)
export const engineData = writable(undefined)
export const frame = writable(0)

export async function makeEngine(newProject: any, newData: any) {
	const newEngine = new Engine(newProject, newData.project, false)

	await newEngine.load()

	engine.set(newEngine)
	engineProject.set(engineProject)
	engineData.set(newData)
}
