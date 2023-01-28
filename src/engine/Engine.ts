import { Runtime } from '@/Runtime'
import { Vector } from '@/engine/engine-core'

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

    scenes(scenes: { [key: string]: string }) {
      engine.scenes = scenes
    },

    initialScene(initialScene: string) {
      engine.initialScene = initialScene
    },

    audioTrack(path: string) {
      if (forReload) return

      engine.loadAudioTrack(path)
    },
  }
}

class Element {
  [key: string]: any

  async render(ctx: any) {
    console.warn('Element has no render!')
  }

  constructor(builder: any, options: any) {
    if (builder == undefined) return

    builder.setup(this, options)

    this.render = builder.initRender(this)
  }
}

function useSceneContext(scene: Scene) {
  return {
    Vector,

    Builders: {
      Rect: {
        setup(element: Element, options: any) {
          element.position = new Vector(0, 0)
          element.origin = new Vector(0.5, 0.5)
          element.size = new Vector(100, 100)
          element.color = new Vector(0, 0, 0, 1)
          element.rotation = 0
          element.priority = 0

          if (options != null) {
            for (const option of Object.keys(options)) {
              element[option] = options[option]
            }
          }
        },

        initRender(me: Element) {
          return async (ctx: any) => {
            const red = me.color.x * 255
            const blue = me.color.y * 255
            const green = me.color.z * 255
            const alpha = me.color.w

            ctx.translate(
              me.position.x + me.size.x / 2,
              me.position.y + me.size.y / 2
            )
            ctx.rotate((me.rotation * Math.PI) / 180)

            ctx.translate(
              -me.position.x + -me.size.x / 2,
              -me.position.y + -me.size.y / 2
            )

            ctx.fillStyle = `rgba(${red},${blue},${green},${alpha})`
            ctx.fillRect(
              me.position.x - me.size.x * me.origin.x,
              me.position.y - me.size.y * me.origin.y,
              me.size.x,
              me.size.y
            )
          }
        },
      },
      Circle: {
        setup(element: Element, options: any) {
          element.position = new Vector(0, 0)
          element.origin = new Vector(0.5, 0.5)
          element.size = 50
          element.color = new Vector(0, 0, 0, 1)
          element.priority = 0

          if (options != null) {
            for (const option of Object.keys(options)) {
              element[option] = options[option]
            }
          }
        },

        initRender(me: Element) {
          return async (ctx: any) => {
            const red = me.color.x * 255
            const blue = me.color.y * 255
            const green = me.color.z * 255
            const alpha = me.color.w

            ctx.fillStyle = `rgba(${red},${blue},${green},${alpha})`
            ctx.beginPath()
            ctx.arc(
              me.position.x - me.size * 2 * (me.origin.x - 0.5),
              me.position.y - me.size * 2 * (me.origin.y - 0.5),
              me.size,
              0,
              2 * Math.PI
            )
            ctx.fill()
          }
        },
      },
      Image: {
        setup(element: Element, options: any) {
          element.position = new Vector(0, 0)
          element.origin = new Vector(0.5, 0.5)
          element.size = new Vector(100, 100)
          element.rotation = 0
          element.priority = 0
          element.image = null

          if (options != null) {
            for (const option of Object.keys(options)) {
              element[option] = options[option]
            }
          }
        },

        initRender(me: Element) {
          return async (ctx: any) => {
            if (me.image == null) return

            const xScale = me.image.width / me.size.x
            const yScale = me.image.height / me.size.y

            let scale = xScale < yScale ? xScale : yScale

            const targetW = me.image.width / xScale
            const targetH = me.image.height / yScale

            const uncroppedW = me.image.width / scale
            const uncroppedH = me.image.height / scale

            const offsetX = ((uncroppedW - targetW) * scale) / 2
            const offsetY = ((uncroppedH - targetH) * scale) / 2

            ctx.translate(
              me.position.x + me.size.x / 2,
              me.position.y + me.size.y / 2
            )
            ctx.rotate((me.rotation * Math.PI) / 180)

            ctx.translate(
              -me.position.x + -me.size.x / 2,
              -me.position.y + -me.size.y / 2
            )

            ctx.translate(0, targetH)
            ctx.scale(1, -1)

            ctx.drawImage(
              me.image,
              offsetX,
              offsetY,
              me.image.width - offsetX * 2,
              me.image.height - offsetY * 2,
              me.position.x - me.size.x * me.origin.x,
              -me.position.y + me.size.y * me.origin.y,
              targetW,
              targetH
            )
          }
        },
      },
    },

    Modes: {
      Linear(time: number): number {
        return time
      },
      Ease(time: number): number {
        return time < 0.5 ? 2 * time * time : 1 - Math.pow(-2 * time + 2, 2) / 2
      },
      EaseIn(time: number): number {
        return Math.pow(time, 3)
      },
      EaseOut(time: number): number {
        return 1 - Math.pow(1 - time, 3)
      },
    },

    Transitions: {
      Cut: function* ({ load, unload }: any) {
        unload()
        load()
      },

      Fade(length: number) {
        return function* ({ load, unload, defineModifier }: any) {
          load()

          let time = 0

          defineModifier((render: CanvasImageSource) => {
            const canvas = document.createElement('canvas')
            canvas.width = 1920
            canvas.height = 1080
            const ctx = canvas.getContext('2d')!

            ctx.globalAlpha = time
            ctx.drawImage(render, 0, 0)

            return canvas
          })

          for (
            let frame = 1;
            frame <= Math.ceil(length * scene.engine.frameRate);
            frame++
          ) {
            time = frame / Math.ceil(length * scene.engine.frameRate)

            yield null
          }

          unload()
        }
      },
    },

    Element,

    addElement(element: Element) {
      scene.addElement(element)

      return element
    },

    animate: async function* (length: number, mode: any, operator: any) {
      for (let i = 1; i <= Math.ceil(length * scene.engine.frameRate); i++) {
        await operator(mode(i / Math.ceil(length * scene.engine.frameRate)))

        yield null
      }
    },

    animateVector: function* (
      property: any,
      a: Vector,
      b: Vector,
      length: number,
      mode: any
    ) {
      const aCopy = new Vector(a.x, a.y, a.z, a.w)
      const bCopy = new Vector(b.x, b.y, b.z, b.w)

      for (let i = 1; i <= Math.ceil(length * scene.engine.frameRate); i++) {
        const frameResult = aCopy.lerp(
          bCopy,
          mode(i / Math.ceil(length * scene.engine.frameRate))
        )

        property.x = frameResult.x
        property.y = frameResult.y
        property.z = frameResult.z
        property.w = frameResult.w

        yield null
      }
    },

    wait: function* (length: number) {
      for (let i = 1; i <= Math.ceil(length * scene.engine.frameRate); i++) {
        yield null
      }
    },

    waitForMarker: function* (name: string) {
      while (
        !scene.engine.markers.find(
          marker => marker.name == name && marker.frame == scene.engine.frame
        )
      ) {
        yield null
      }
    },

    waitWhile: async function* (condition: any) {
      while (await condition()) yield null
    },

    lerp(a: number, b: number, t: number) {
      return a + (b - a) * t
    },

    relative(pos: Vector) {
      return new Vector(pos.x * 1920, pos.y * 1080, pos.z, pos.w)
    },

    absolute(pos: Vector) {
      return new Vector(pos.x / 1920, pos.y / 1080, pos.z, pos.w)
    },

    seconds(seconds: number) {
      return Math.ceil(seconds * scene.engine.frameRate)
    },

    minutes(minutes: number) {
      return minutes * 60
    },

    minutesFrames(minutes: number) {
      return Math.ceil(minutes * 60 * scene.engine.frameRate)
    },

    frames(frames: number) {
      return frames / scene.engine.frameRate
    },

    waitForAll: async function* () {
      let contexts = []

      for (let i = 0; i < arguments.length; i++) {
        contexts.push(arguments[i])
      }

      let allDone = false

      while (!allDone) {
        allDone = true

        for (let i = 0; i < contexts.length; i++) {
          const context = contexts[i]

          const result = await context.next()

          if (result.done) {
            contexts.splice(i, 1)
            i--
          } else {
            allDone = false
          }
        }

        if (!allDone) {
          yield null
        }
      }
    },

    waitForAny: async function* () {
      let contexts = []

      for (let i = 0; i < arguments.length; i++) {
        const context = arguments[i]

        contexts.push(context)
      }

      let anyDone = false

      while (!anyDone) {
        for (let i = 0; i < contexts.length; i++) {
          const context = contexts[i]

          const result = await context.next()

          if (result.done) {
            contexts.splice(i, 1)
            i--

            anyDone = true
          }
        }

        if (!anyDone) {
          yield null
        }
      }

      for (const context of contexts) {
        yield context
      }
    },

    loop: async function* (contextLambda: any) {
      while (true) {
        // may not need to be async
        const context = await contextLambda()

        yield* context
      }
    },

    transition: async function* (name: string, transition: any) {
      const targetScene = new Scene(scene.engine.scenes[name], scene.engine)
      await targetScene.load()

      yield* transition({
        load() {
          scene.engine.activeScenes.push(targetScene)
        },
        unload() {
          scene.engine.activeScenes.splice(
            scene.engine.activeScenes.findIndex(s => s == scene),
            1
          )
        },
        defineModifier(modifier: any) {
          targetScene.transitionRenderModifier = modifier
        },
      })
    },
  }
}

function isGenerator(obj: any) {
  if (!obj) return false

  if (!obj.constructor) return false

  if (!obj.constructor.constructor) return false

  if (!obj.constructor.constructor.name) return false

  return (
    obj.constructor.constructor.name == 'GeneratorFunction' ||
    obj.constructor.constructor.name == 'AsyncGeneratorFunction'
  )
}

class Scene {
  context: any
  path: string
  engine: Engine
  elements: Element[] = []
  sideContexts: any[] = []
  transitionRenderModifier: any = null

  constructor(path: string, engine: Engine) {
    this.path = path
    this.engine = engine
  }

  async load() {
    this.context = (await this.engine.runtime.run(this.path)).scene(
      useSceneContext(this)
    )

    await this.next()
  }

  async render() {
    const canvas = document.createElement('canvas')
    canvas.width = 1920
    canvas.height = 1080
    const ctx = canvas.getContext('2d')!

    ctx.fillStyle = '#ffffff'
    ctx.fillRect(0, 0, 1920, 1080)

    let sortedElements = this.elements.sort((a: any, b: any) => {
      return (a.priority || 0) - (b.priority || 0)
    })

    for (const element of sortedElements) {
      ctx.translate(0, 1080)
      ctx.scale(1, -1)

      await element.render(ctx)

      ctx.resetTransform()
    }

    if (this.transitionRenderModifier)
      return this.transitionRenderModifier(canvas)

    return canvas
  }

  async next() {
    for (let i = 0; i < this.sideContexts.length; i++) {
      const generator = this.sideContexts[i]

      await generator.next()

      if (generator.done == 'true') {
        this.sideContexts.splice(i, 1)

        i--
      }
    }

    let result = (await this.context.next()).value

    while (result != null && result != undefined && isGenerator(result)) {
      await result.next()

      this.sideContexts.push(result)

      result = (await this.context.next()).value
    }
  }

  addElement(element: Element) {
    this.elements.push(element)
  }
}

export class Engine {
  project: any
  runtime: Runtime

  scenes: {
    [key: string]: string
  } = {}
  initialScene: undefined | string = undefined
  activeScenes: Scene[] = []

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

  constructor(
    runtime: Runtime,
    markers: { name: string; id: string; frame: number }[],
    inferenceAudio?: boolean
  ) {
    this.runtime = runtime
    this.markers = markers

    if (inferenceAudio) this.inferenceAudio = inferenceAudio
  }

  async load() {
    this.frameRate = 60
    this.length = 60
    this.scenes = {}
    this.initialScene = undefined

    this.project = await this.runtime.run('project.ts')

    this.project.project(useProjectContext(this))

    this.frame = 0

    if (!this.initialScene) return

    this.activeScenes = [new Scene(this.scenes[this.initialScene], this)]
    await this.activeScenes[0].load()
  }

  async render() {
    const canvas = document.createElement('canvas')
    canvas.width = 1920
    canvas.height = 1080
    const ctx = canvas.getContext('2d')!

    for (const scene of this.activeScenes) {
      const activeSceneRender = await scene.render()

      ctx.drawImage(activeSceneRender, 0, 0)
    }

    return canvas
  }

  async reloadContext() {
    this.frameRate = 60
    this.length = 60
    this.scenes = {}
    this.initialScene = undefined

    this.project.project(useProjectContext(this, true))

    this.frame = 0

    if (!this.initialScene) return

    this.activeScenes = [new Scene(this.scenes[this.initialScene], this)]

    await this.activeScenes[0].load()
  }

  async next() {
    try {
      for (const scene of this.activeScenes) {
        await scene.next()
      }
    } catch (err) {
      console.error(err)
    }

    this.frame++
  }

  async loadAudioTrack(path: string) {
    if (!this.inferenceAudio) return

    const audioFile = await this.runtime.readFile(path)

    const ctx = new AudioContext()

    if (!ctx) return

    const audioBuffer = await ctx.decodeAudioData(await audioFile.arrayBuffer())

    this.audioBuffer = audioBuffer

    const channels = []

    for (
      let channelIndex = 0;
      channelIndex < this.audioBuffer.numberOfChannels;
      channelIndex++
    ) {
      channels.push(this.audioBuffer.getChannelData(channelIndex))
    }

    const worker = this.runtime.createAudioInferenceWorker()

    worker.onmessage = message => {
      this.volumePerFrame = message.data

      ctx.close()
    }

    worker.postMessage({
      sampleRate: this.audioBuffer.sampleRate,
      frameRate: this.frameRate,
      length: this.length,
      channels,
    })
  }
}
