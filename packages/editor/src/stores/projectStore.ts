import type { Writable } from 'svelte/store'

import { writable } from 'svelte/store'
import { Scene } from '@vector-engine/core'

export const scenes: Writable<{ [key: string]: Scene }> = writable({})

document.addEventListener('@vector-engine/project-reload', (event: CustomEvent) => {
	scenes.set(event.detail.scenes)
})
