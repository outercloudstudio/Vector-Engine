export function clip(context: () => Generator<null, never, unknown>)

interface Element {}

export function add(element: Element)

export class Rect implements Element {
	position: Vector2
	size: Vector2

	constructor(position: Vector2, size: Vector2)
}

export class Vector2 implements Element {
	x: number
	y: number

	constructor(x: number, y: number)
}
