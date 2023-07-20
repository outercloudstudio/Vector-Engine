import { writable, type Writable } from 'svelte/store'

export type Selectable = {
	type: string
	content: any
	origin?: any
}

export const selected: Writable<Selectable | null> = writable(null)
