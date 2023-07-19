import { writable, type Writable } from 'svelte/store'
import { type Meta, Asset } from '@vector-engine/core'

export const assets: Writable<{ [key: string]: () => Asset }> = writable({})
export const meta: Writable<Meta> = writable({
	assets: {},
})

document.addEventListener('@vector-engine/project-reload', (event: CustomEvent) => {
	console.log(event.detail.assets)

	assets.set(event.detail.assets)
	meta.set(event.detail.meta)
})
