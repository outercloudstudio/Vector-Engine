import { Scene, SceneContext } from './Scene'

export function makeProject(
	frameRate: number,
	length: number,
	scene: {
		name: string
		context: (context: SceneContext) => AsyncGenerator
	},
	audioTrack: AudioBuffer | undefined
) {
	return {
		frameRate,
		length: length * frameRate,
		scene,
		audioTrack,
	}
}

export class Engine {
	project: any
	data: {
		markers: {
			name: string
			id: string
			frame: number
		}[]
	}
	loaded: boolean = false

	currentScene: Scene

	frameRate: number = 60
	length: number = 60
	frame: number = 0

	audioTrack: AudioBuffer | undefined = undefined

	constructor(project: any, data: any) {
		this.project = project
		this.data = data
		this.frameRate = project.frameRate
		this.length = project.length
	}

	async load() {
		this.currentScene = new Scene(this.project.scene, this)
		await this.currentScene.load()

		this.loaded = true
	}

	async render() {
		const canvas = new OffscreenCanvas(1920, 1080)
		const ctx: OffscreenCanvasRenderingContext2D = <OffscreenCanvasRenderingContext2D>(
			canvas.getContext('2d')
		)

		const activeSceneRender = await this.currentScene.render()

		ctx.drawImage(activeSceneRender, 0, 0)

		return canvas
	}

	async jumpToFrame(frame: number) {
		if (frame < this.frame) {
			this.frame = 0

			this.currentScene = new Scene(this.project.scene, this)
			await this.currentScene.load()
		}

		for (let engineFrame = this.frame; engineFrame < frame; engineFrame++) {
			await this.next()
		}
	}

	async next() {
		this.frame++

		await this.currentScene.next()
	}

	referencedMarker(name: string): void {}
}
