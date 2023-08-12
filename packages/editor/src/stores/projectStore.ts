import { get, writable, type Writable } from 'svelte/store'
import { type Meta, Asset, type Project } from '@vector-engine/core'

export const assets: Writable<{ [key: string]: () => Asset }> = writable({})
export const meta: Writable<Meta> = writable({
	assets: {},
})

export const project: Writable<Project> = writable({
	timeline: {},
})

document.addEventListener('@vector-engine/project-reload', (event: CustomEvent) => {
	assets.set(event.detail.assets)

	// We have to use default since it's not a virtual store for some reason
	meta.set(event.detail.meta.default)
	project.set(event.detail.project.default)
})

export function saveProject() {
	console.log('saving project')
	console.log(get(project))

	import.meta.hot.send('@vector-engine/update-project', get(project))
}
