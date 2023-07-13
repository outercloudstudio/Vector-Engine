import { get, writable, type Writable } from 'svelte/store'

export type Holdable = {
	type: string
	content: any
	origin?: any
}

export const heldX: Writable<number> = writable(0)
export const heldY: Writable<number> = writable(0)

export const dropped: Writable<Holdable | null> = writable(null)

export const held: Writable<Holdable | null> = writable(null, set => {
	function updateHeldPosition(event: MouseEvent) {
		heldX.set(event.clientX)
		heldY.set(event.clientY)
	}

	function dropHeld(event: MouseEvent) {
		if (get(held) === null) return

		dropped.set(get(held))

		held.set(null)
	}

	window.addEventListener('mousemove', updateHeldPosition)
	window.addEventListener('mouseup', dropHeld)

	return () => {
		window.removeEventListener('mousemove', updateHeldPosition)
		window.removeEventListener('mouseup', dropHeld)
	}
})

export function hold(holdable: Holdable) {
	held.set(holdable)
}

export function droppedOn(element: HTMLElement): boolean {
	const bounds = element.getBoundingClientRect()

	const x = get(heldX)
	const y = get(heldY)

	return (
		x >= bounds.x && x < bounds.x + bounds.width && y >= bounds.y && y < bounds.y + bounds.height
	)
}
