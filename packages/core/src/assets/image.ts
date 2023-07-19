import { Asset } from './asset'

export function image(path: string): () => ImageClip {
	return () => new ImageClip(path)
}

export class ImageClip extends Asset {
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
		context.drawImage(this.image, 1920 / 2 - this.image.width / 2, 1080 / 2 - this.image.height / 2)
	}
}
