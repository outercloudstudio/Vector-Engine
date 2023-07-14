import type { Writable } from 'svelte/store'

import { writable } from 'svelte/store'
import { Scene } from '@vector-engine/core'

export const assets: Writable<{ [key: string]: Scene }> = writable({})

document.addEventListener('@vector-engine/project-reload', (event: CustomEvent) => {
	console.log(event.detail)

	assets.set(event.detail.assets)
})
