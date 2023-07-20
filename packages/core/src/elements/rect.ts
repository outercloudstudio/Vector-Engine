import { Vector2 } from '../vector'
import { Element } from './element'

export class Rect extends Element {
	position: Vector2
	size: Vector2

	constructor(position: Vector2, size: Vector2) {
		super()

		this.position = position
		this.size = size
	}

	async render(canvas: OffscreenCanvas) {
		const context = canvas.getContext('2d')

		context.fillStyle = 'black'
		context.fillRect(this.position.x + 1920 / 2, this.position.y + 1080 / 2, this.size.x, this.size.y)
	}
}
