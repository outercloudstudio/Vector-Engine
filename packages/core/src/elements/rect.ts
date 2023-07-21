import { Vector2 } from '../vector'
import { AnimatedReactiveProperty, Element, animatedReactiveProperty } from './element'

export class Rect extends Element {
	public position: Vector2

	public size: AnimatedReactiveProperty<Vector2> = animatedReactiveProperty<Vector2>(Vector2.zero())

	constructor(position: Vector2, size: Vector2) {
		super()

		this.position = position
		this.size(size)
	}

	public async render(canvas: OffscreenCanvas) {
		const context = canvas.getContext('2d')

		const size = this.size()

		context.fillStyle = 'black'
		context.fillRect(this.position.x + 1920 / 2, this.position.y + 1080 / 2, size.x, size.y)
	}
}
