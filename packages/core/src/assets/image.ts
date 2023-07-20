import { Vector2 } from '../vector'
import { Asset } from './asset'

export function image(path: string): () => ImageAsset {
	return () => new ImageAsset(path)
}

export class ImageAsset extends Asset {
	position: Vector2
	size: Vector2

	private image: HTMLImageElement | null = null
	private path: string

	constructor(path: string) {
		super()

		this.path = path
	}

	async render(canvas: OffscreenCanvas) {
		if (this.image === null)
			this.image = await new Promise<HTMLImageElement>(res => {
				const image = new Image()

				image.addEventListener('load', () => res(image))

				image.src = `/@asset?type=image&path=${encodeURI(this.path)}`
			})

		const context = canvas.getContext('2d')
		context.drawImage(
			this.image,
			1920 / 2 - this.size.x / 2 + this.position.x,
			1080 / 2 - this.size.y / 2 + this.position.y,
			this.size.x,
			this.size.y
		)
	}
}
