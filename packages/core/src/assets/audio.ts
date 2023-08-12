import { Asset } from './asset'

export function audio(path: string): () => AudioAsset {
	return () => new AudioAsset(path)
}

export class AudioAsset extends Asset {
	private audio: HTMLAudioElement | null = null
	private path: string
	private loading: boolean = false
	private internalFrame: number

	constructor(path: string) {
		super()

		this.path = path
	}

	async loadAudio() {
		if (this.audio === null) {
			if (this.loading) {
				let intervalId: number = 0

				await new Promise<void>(res => {
					intervalId = <any>setInterval(() => {
						if (!this.loading) res()
					}, 1)
				})

				clearInterval(intervalId)

				return
			}

			this.loading = true

			this.audio = await new Promise<HTMLAudioElement>(res => {
				const audio = new Audio()

				audio.addEventListener('canplaythrough', () => res(audio))

				audio.src = `/@asset?type=audio&path=${encodeURI(this.path)}`
			})

			this.loading = false

			console.warn('Loaded Audio')
		}
	}

	async waitForReady() {
		let intervalId: number = 0

		await new Promise<void>(res => {
			intervalId = <any>setInterval(() => {
				if (this.audio.readyState === HTMLMediaElement.HAVE_ENOUGH_DATA) res()
			}, 1)
		})

		clearInterval(intervalId)
	}

	async renderAudio(frame: number, length: number) {
		await this.loadAudio()
	}

	async previewAudio(frame: number) {
		this.internalFrame = frame

		await this.loadAudio()

		this.audio.play()

		if (Math.abs(frame / 60 - this.audio.currentTime) > 0.3) {
			this.audio.currentTime = frame / 60
		}

		setTimeout(() => {
			if (frame !== this.internalFrame) return

			this.audio.pause()
		}, (1 / 60) * 1000 + 2)
	}
}
