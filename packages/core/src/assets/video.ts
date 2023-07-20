import { Asset } from './asset'

export function video(path: string): () => VideoAsset {
	return () => new VideoAsset(path)
}

export class VideoAsset extends Asset {
	private video: HTMLVideoElement | null = null
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
		if (this.video === null) {
			this.video = document.createElement('video')
			this.video.autoplay = true
			this.video.muted = true
			this.video.pause()
			this.video.src = `/@asset?type=video&path=${encodeURI(this.path)}`

			let intervalId: number = 0

			await new Promise<void>(res => {
				intervalId = <any>setInterval(() => {
					if (this.video.readyState === 4) res()
				}, 1)
			})

			clearInterval(intervalId)

			console.warn('Loaded Video')
		}

		if (this.frames[this.internalFrame] === undefined) {
			const frameCanvas = new OffscreenCanvas(this.video.videoWidth, this.video.videoHeight)
			const context = frameCanvas.getContext('2d')

			this.video.currentTime = this.internalFrame / 60

			await new Promise(res => this.video.addEventListener('seeked', res))

			context.drawImage(
				this.video,
				1920 / 2 - this.video.videoWidth / 2,
				1080 / 2 - this.video.videoHeight / 2
			)

			const fileReader = new FileReader()

			const dataUrl = await new Promise<string>(async res => {
				fileReader.onload = event => res(event.target.result.toString())

				fileReader.readAsDataURL(await frameCanvas.convertToBlob())
			})

			const image = await new Promise<HTMLImageElement>(res => {
				const image = new Image()

				image.addEventListener('load', () => res(image))

				image.src = dataUrl
			})

			this.frames[this.internalFrame] = image
		}

		const context = canvas.getContext('2d')
		context.drawImage(
			this.frames[this.internalFrame],
			1920 / 2 - this.frames[this.internalFrame].width / 2,
			1080 / 2 - this.frames[this.internalFrame].height / 2
		)
	}
}
