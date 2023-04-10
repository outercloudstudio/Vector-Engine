import { Scene } from './Scene'

function useProjectContext(engine: Engine, forReload?: boolean) {
  return {
    frameRate(frameRate: number) {
      engine.frameRate = frameRate
    },

    length(length: number) {
      engine.length = length
    },

    minutes(minutes: number) {
      return engine.frameRate * 60 * minutes
    },

    seconds(seconds: number) {
      return engine.frameRate * seconds
    },

    loadScenes(scenes: { [key: string]: any }) {
      engine.sceneContexts = scenes
    },

    audioTrack(audio: any) {
      if (forReload) return

      engine.audioTrack = audio
    },
  }
}

export class Engine {
  project: any
  loaded: boolean = false

  sceneContexts: { [key: string]: any } = {}
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

    for (
      let stepFrame = 0;
      stepFrame <= frame && sceneIndex < this.scenes.length - 1;

    ) {
      sceneIndex++

      stepFrame += this.scenes[sceneIndex].length
    }

    return sceneIndex
  }

  getSceneStartFrame(sceneIndex: number): number {
    let frame = 0

    for (let index = 0; index < sceneIndex; index++) {
      frame += this.scenes[sceneIndex].length
    }

    return frame
  }

  async loadScene(sceneIndex: number) {
    const sceneData = this.scenes[sceneIndex]

    this.currentScene = new Scene(this.sceneContexts[sceneData.name], this)
    await this.currentScene.load()

    this.currentSceneIndex = sceneIndex
  }

  async load() {
    try {
      await this.project(useProjectContext(this))
    } catch (error) {
      if (this.onError) this.onError(<string>error)

      return
    }

    await this.loadScene(0)

    this.loaded = true
  }

  async render() {
    const canvas = new OffscreenCanvas(1920, 1080)
    const ctx: OffscreenCanvasRenderingContext2D = <
      OffscreenCanvasRenderingContext2D
    >canvas.getContext('2d')

    const activeSceneRender = await this.currentScene.render()

    ctx.drawImage(activeSceneRender, 0, 0)

    return canvas
  }

  async reload() {
    this.loaded = false
    this.frameRate = 60
    this.length = 60
    this.sceneContexts = {}
    this.frame = 0

    if (!this.project) return

    try {
      await this.project(useProjectContext(this))
    } catch (error) {
      if (this.onError) this.onError(<string>error)

      return
    }

    this.loaded = true
  }

  async jumpToFrame(frame: number) {
    if (frame < this.frame) {
      await this.reload()

      const sceneIndex = this.getSceneIndex(frame)
      const sceneStartFrame = this.getSceneStartFrame(sceneIndex)

      await this.loadScene(sceneIndex)

      this.frame = sceneStartFrame
    }

    for (let engineFrame = this.frame; engineFrame < frame; engineFrame++) {
      await this.next()
    }
  }

  async next() {
    this.frame++

    await this.currentScene.next()

    const currentSceneIndex = this.getSceneIndex(this.frame)

    if (currentSceneIndex != this.currentSceneIndex)
      await this.loadScene(currentSceneIndex)
  }
}
