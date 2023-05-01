import { Scene, SceneContext } from './Scene'

export function makeProject(
	frameRate: number,
	length: number,
	scenes: {
		[key: string]: (context: SceneContext) => AsyncGenerator
	},
	audioTrack: AudioBuffer | undefined
) {
	return {
		frameRate,
		length: length * frameRate,
		scenes,
		audioTrack,
	}
}

export class Engine {
	project: any
	loaded: boolean = false

	scenes: { name: string; length: number }[] = []
	currentScene: Scene
	currentSceneIndex: number = -1

	frameRate: number = 60
	length: number = 60
	frame: number = 0

	markers: {
		name: string
		id: string
		frame: number
	}[] = []

	audioTrack: AudioBuffer | undefined = undefined

	inferenceAudio: boolean = false

	onError: any

	constructor(
		project: any,
		scenes: { name: string; length: number }[],
		markers: { name: string; id: string; frame: number }[],
		inferenceAudio?: boolean,
		onError?: any
	) {
		this.project = project

		this.scenes = scenes

		this.markers = markers

		if (inferenceAudio) this.inferenceAudio = inferenceAudio

		this.onError = onError
	}

	getSceneIndex(frame: number): number {
		let sceneIndex = -1

		for (let stepFrame = 0; stepFrame <= frame && sceneIndex < this.scenes.length - 1; ) {
			sceneIndex++

			stepFrame += this.scenes[sceneIndex].length
		}

		return sceneIndex
	}

	getSceneStartFrame(sceneIndex: number): number {
		let frame = 0

		for (let index = 0; index < sceneIndex; index++) {
			frame += this.scenes[index].length
		}

		return frame
	}

	async loadScene(sceneIndex: number) {
		const sceneData = this.scenes[sceneIndex]

		this.currentScene = new Scene(this.project.scenes[sceneData.name], this)
		await this.currentScene.load()

		this.currentSceneIndex = sceneIndex
	}

	async load() {
		await this.loadScene(0)

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

			const sceneIndex = this.getSceneIndex(frame)
			const sceneStartFrame = this.getSceneStartFrame(sceneIndex)

			this.frame = sceneStartFrame

			await this.loadScene(sceneIndex)
		}

		for (let engineFrame = this.frame; engineFrame < frame; engineFrame++) {
			await this.next()
		}
	}

	async next() {
		this.frame++

		await this.currentScene.next()

		const currentSceneIndex = this.getSceneIndex(this.frame)

		if (currentSceneIndex != this.currentSceneIndex) await this.loadScene(currentSceneIndex)
	}
}
