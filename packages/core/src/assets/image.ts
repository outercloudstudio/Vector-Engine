import { Element } from '../elements/element'
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
		if (this.image === null) {
			const buffer = await new Promise<ArrayBufferLike>(res => {
				//@ts-ignore
				import.meta.hot.on('@vector-engine/image', data => {
					res(new Uint8Array(data.data).buffer)
				})

				//@ts-ignore
				import.meta.hot.send('@vector-engine/image', { path: this.path })
			})

			const blob = new Blob([buffer], { type: this.path.endsWith('.png') ? 'image/png' : 'image/jpg' })
			const url = URL.createObjectURL(blob)

			const image = await new Promise<HTMLImageElement>(res => {
				const image = new Image()

				image.addEventListener('load', () => res(image))

				image.src = url
			})

			URL.revokeObjectURL(url)

			this.image = image
		}

		const context = canvas.getContext('2d')
		context.drawImage(this.image, 0, 0)
	}
}
