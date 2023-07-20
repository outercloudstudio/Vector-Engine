import { ImageAsset } from '../assets/image'
import { Vector2 } from '../vector'
import { Element } from './element'

export class VectorImage extends Element {
	image: ImageAsset
	position: Vector2
	size: Vector2

	constructor(image: () => ImageAsset, position: Vector2, size: Vector2) {
		super()

		this.image = image()
		this.position = position
		this.size = size
	}

	async render(canvas: OffscreenCanvas) {
		this.image.position = this.position
		this.image.size = this.size

		await this.image.render(canvas)
	}
}
