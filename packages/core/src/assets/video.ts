import { Asset } from './asset'

export function video(path: string): () => VideoClip {
	return () => new VideoClip(path)
}

export class VideoClip extends Asset {
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
		if (this.video === null)
			this.video = await new Promise<HTMLVideoElement>(res => {
				const video = document.createElement('video')

				video.addEventListener('loadeddata', () => res(video))

				video.src = `/@asset?type=video&path=${encodeURI(this.path)}`
			})

		if (this.frames[this.internalFrame] === undefined) {
			console.log(this.video.videoWidth, this.video.videoHeight)

			const frameCanvas = new OffscreenCanvas(this.video.videoWidth, this.video.videoHeight)
			const context = frameCanvas.getContext('2d')

			this.video.currentTime = this.internalFrame / 60

			await new Promise(res => this.video.addEventListener('seeked', res))

			context.drawImage(
				this.video,
				1920 / 2 - this.video.videoWidth / 2,
				1080 / 2 - this.video.videoHeight / 2
			)

			const image = new Image()

			const fileReader = new FileReader()

			image.src = await new Promise<string>(async res => {
				fileReader.onload = event => res(event.target.result.toString())

				fileReader.readAsDataURL(await frameCanvas.convertToBlob())
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
