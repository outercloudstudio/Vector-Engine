import { Asset } from './asset'

export function video(path: string): () => VideoClip {
	return () => new VideoClip(path)
}

export class VideoClip extends Asset {
	private frames: HTMLImageElement[] = []
	private path: string

	private internalFrame: number

	constructor(path: string) {
		super()

		this.path = path
	}

	toFrame(frame: number): void {
		this.internalFrame = frame
	}

	async render(canvas: OffscreenCanvas) {
		if (this.frames[this.internalFrame] === null) {
			const buffer = await new Promise<ArrayBufferLike>(res => {
				//@ts-ignore
				import.meta.hot.on('@vector-engine/video', data => {
					res(new Uint8Array(data.data).buffer)
				})

				//@ts-ignore
				import.meta.hot.send('@vector-engine/video', { path: this.path })
			})

			const blob = new Blob([buffer], { type: 'image/jpg' })
			const url = URL.createObjectURL(blob)

			const image = await new Promise<HTMLImageElement>(res => {
				const image = new Image()

				image.addEventListener('load', () => res(image))

				image.src = url
			})

			URL.revokeObjectURL(url)

			this.frames[this.internalFrame] = image
		}

		const frame = this.frames[this.internalFrame]

		const context = canvas.getContext('2d')
		context.drawImage(frame, 1920 / 2 - frame.width / 2, 1080 / 2 - frame.height / 2)
	}
}
