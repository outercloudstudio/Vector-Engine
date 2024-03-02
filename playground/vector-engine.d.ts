declare function clip(context: () => Generator<null, never, unknown>)

declare interface VectorElement {}

declare function add<T extends VectorElement>(element: T): T

declare class Rect implements VectorElement {
	position: Vector2
	size: Vector2

	constructor(position: Vector2, size: Vector2)
}

declare class Vector2 implements VectorElement {
	x: number
	y: number

	constructor(x: number, y: number)
}
