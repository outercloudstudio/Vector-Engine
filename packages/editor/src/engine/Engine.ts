import { Scene } from '@/engine/Scene'

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

    async loadScene(sceneContext: any) {
      const scene = new Scene(sceneContext, engine)
      await scene.load()
      engine.scenes.push(scene)
    },

    audioTrack(path: string) {
      if (forReload) return

      engine.loadAudioTrack(path)
    },
  }
}

export class Engine {
  project: any
  loaded: boolean = false

  scenes: Scene[] = []

  frameRate: number = 60
  length: number = 60
  frame: number = -1

  markers: {
    name: string
    id: string
    frame: number
  }[] = []

  volumePerFrame: number[] = []
  audioBuffer: AudioBuffer | null = null

  inferenceAudio: boolean = false

  errorListener: any

  constructor(
    project: any,
    markers: { name: string; id: string; frame: number }[],
    inferenceAudio?: boolean,
    errorListener?: any
  ) {
    this.project = project

    this.markers = markers

    if (inferenceAudio) this.inferenceAudio = inferenceAudio

    this.errorListener = errorListener
  }

  async load() {
    try {
      await this.project(useProjectContext(this))
    } catch (error) {
      console.error(error)

      if (this.errorListener) this.errorListener(<string>error)

      return
    }

    this.loaded = true
  }

  async render() {
    const canvas = new OffscreenCanvas(1920, 1080)
    const ctx: OffscreenCanvasRenderingContext2D = <
      OffscreenCanvasRenderingContext2D
    >canvas.getContext('2d')

    for (const scene of this.scenes) {
      const activeSceneRender = await scene.render()

      ctx.drawImage(activeSceneRender, 0, 0)
    }

    return canvas
  }

  async reload() {
    this.frameRate = 60
    this.length = 60
    this.scenes = []
    this.frame = 0

    if (!this.project) return

    try {
      await this.project(useProjectContext(this))
    } catch (error) {
      console.error(error)

      if (this.errorListener) this.errorListener(<string>error)

      return
    }
  }

  async next() {
    for (const scene of this.scenes) {
      await scene.next()
    }

    this.frame++
  }

  async loadAudioTrack(path: string) {
    if (!this.inferenceAudio) return

    // const audioFile = await this.runtime.readFile(path)

    // const ctx = new AudioContext()

    // if (!ctx) return

    // const audioBuffer = await ctx.decodeAudioData(await audioFile.arrayBuffer())

    // this.audioBuffer = audioBuffer

    // const channels = []

    // for (
    //   let channelIndex = 0;
    //   channelIndex < this.audioBuffer.numberOfChannels;
    //   channelIndex++
    // ) {
    //   channels.push(this.audioBuffer.getChannelData(channelIndex))
    // }

    // const worker = this.runtime.createAudioInferenceWorker()

    // worker.onmessage = message => {
    //   this.volumePerFrame = message.data

    //   ctx.close()
    // }

    // worker.postMessage({
    //   sampleRate: this.audioBuffer.sampleRate,
    //   frameRate: this.frameRate,
    //   length: this.length,
    //   channels,
    // })
  }
}
