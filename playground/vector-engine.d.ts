declare function clip(context: () => Generator<null, never, unknown>)

declare interface VectorElement {}

declare function add<T extends VectorElement>(element: T): T

declare class Rect implements VectorElement {
	position: Vector2
	size: Vector2
	color: Vector4

	constructor(position: Vector2, size: Vector2, color: Vector4)
}

declare class Vector2 implements VectorElement {
	x: number
	y: number

	constructor(x: number, y: number)
}

declare class Vector4 implements VectorElement {
	x: number
	y: number
	z: number
	w: number

	constructor(x: number, y: number, z: number, w: number)
}
