import { Runtime } from '@/Runtime'

function useProjectContext(engine: Engine) {
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

    Element,

    addElement(element: Element) {
      scene.addElement(element)
    },

    animate: function* (length: number, mode: any, operator: any) {
      for (let i = 1; i <= Math.ceil(length * scene.engine.frameRate); i++) {
        operator(mode(i / Math.ceil(length * scene.engine.frameRate)))

        yield null
      }
    },
  }
}

class Scene {
  context: any
  path: string
  engine: Engine
  elements: Element[] = []
  sideContexts: any[] = []

  constructor(path: string, engine: Engine) {
    this.path = path
    this.engine = engine
  }

  async load() {
    this.context = (await this.engine.runtime.run(this.path)).scene(
      useSceneContext(this)
    )
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

    while (
      result != null &&
      result != undefined &&
      typeof result[Symbol.iterator] === 'function'
    ) {
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
  initialScene: undefined | string = undefined
  activeScenes: Scene[] = []
  frameRate: number = 60
  length: number = 60
  scenes: {
    [key: string]: string
  } = {}
  runtime: Runtime
  currentFrame: number = -1
  markers: {
    name: string
    frame: number
  }[] = []

  constructor(runtime: Runtime) {
    this.runtime = runtime
  }

  async load() {
    this.frameRate = 60
    this.length = 60
    this.scenes = {}
    this.initialScene = undefined

    this.project = await this.runtime.run('project.ts')

    this.project.project(useProjectContext(this))

    this.currentFrame = -1

    if (!this.initialScene) return

    this.activeScenes = [new Scene(this.scenes[this.initialScene], this)]
    await this.activeScenes[0].load()

    await this.next()
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

    const engine = this

    this.project.project(useProjectContext(this))

    this.currentFrame = -1

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

    this.currentFrame++
  }
}

import { Vector } from '@/engine/engine-core'

function initAnimateVector(frameRate: number) {
  return function* (
    a: Vector,
    b: Vector,
    length: number,
    mode: any,
    operator: any
  ) {
    for (let i = 1; i <= length * frameRate; i++) {
      operator(a.lerp(b, mode(i / (length * frameRate))))

      yield null
    }
  }
}

function initAnimateNumber(frameRate: number) {
  return function* (
    a: number,
    b: number,
    length: number,
    mode: any,
    operator: any
  ) {
    for (let i = 1; i <= length * frameRate; i++) {
      operator(lerp(a, b, mode(i / (length * frameRate))))

      yield null
    }
  }
}

function initAnimateVectorProperty(frameRate: number) {
  return function* (
    obj: any,
    property: string,
    a: Vector,
    b: Vector,
    length: number,
    mode: any
  ) {
    for (let i = 1; i <= length * frameRate; i++) {
      obj[property] = a.lerp(b, mode(i / (length * frameRate)))

      yield null
    }
  }
}

function initAnimateNumberProperty(frameRate: number) {
  return function* (
    obj: any,
    property: string,
    a: number,
    b: number,
    length: number,
    mode: any
  ) {
    for (let i = 1; i <= length * frameRate; i++) {
      obj[property] = lerp(a, b, mode(i / (length * frameRate)))

      yield null
    }
  }
}

function initTransitionTo(frameRate: number, scene: Scene, engine: Engine) {
  return function (
    sceneName: string,
    length: number,
    mode: any,
    transition: any
  ) {
    const targetScene = engine.scenes.find(s => s.name == sceneName)

    if (targetScene == undefined) {
      console.warn(
        `Could not find target scene '${sceneName}' to transtiion to!`
      )

      return
    }

    if (
      engine.transitionScenes.find(tran => tran.scene == scene) != undefined
    ) {
      console.warn('Scene is already transitioning!')

      return
    }

    engine.transitionScenes.push({
      scene: new Scene(targetScene.content, sceneName, frameRate, engine),
      transition,
      leftFrames: length * frameRate,
      length: length * frameRate,
      mode,
    })
  }
}

const Transition = {
  Cut(sceneCtx: any, targetCtx: any, time: number) {
    targetCtx.drawImage(sceneCtx.canvas, 0, 0)
  },
  Fade(sceneCtx: any, targetCtx: any, time: number) {
    const prevAlpha = targetCtx.globalAlpha

    targetCtx.globalAlpha = 1 - time

    targetCtx.drawImage(sceneCtx.canvas, 0, 0)

    targetCtx.globalAlpha = prevAlpha
  },
}

function* all(contexts: any) {
  if (!Array.isArray(contexts)) contexts = arguments

  let builtContexts = []

  for (let i = 0; i < contexts.length; i++) {
    const context = contexts[i]

    builtContexts.push(context)
  }

  let allDone = false

  while (!allDone) {
    allDone = true

    for (let i = 0; i < builtContexts.length; i++) {
      const context = builtContexts[i]

      const result = context.next()

      if (result.done) {
        builtContexts.splice(i, 1)
        i--
      } else {
        allDone = false
      }
    }

    if (!allDone) {
      yield null
    }
  }
}

function* any(contexts: any) {
  if (!Array.isArray(contexts)) contexts = arguments

  let builtContexts = []

  for (let i = 0; i < contexts.length; i++) {
    const context = contexts[i]

    builtContexts.push(context)
  }

  let anyDone = false

  while (!anyDone) {
    for (let i = 0; i < builtContexts.length; i++) {
      const context = builtContexts[i]

      const result = context.next()

      if (result.done) {
        anyDone = true
      }
    }

    if (!anyDone) {
      yield null
    }
  }
}

function initWait(frameRate: number) {
  return function* (length: number) {
    for (let i = 1; i <= length * frameRate; i++) {
      yield null
    }
  }
}

function* aside(contexts: any) {
  if (!Array.isArray(contexts)) contexts = arguments

  for (let i = 0; i < contexts.length; i++) {
    const context = contexts[i]

    yield* context
  }
}

function* forever(context: any) {
  while (true) {
    yield* context()

    yield null
  }
}

function* waitWhile(condition: any) {
  while (condition()) yield null
}

function initWaitForMarker(
  engine: Engine,
  markers: { name: string; frame: number }[]
) {
  return function* (name: string) {
    const targetMarker = markers.find(marker => marker.name == name)

    while (
      targetMarker == undefined ||
      targetMarker.frame - 1 != engine.currentFrame
    ) {
      yield null
    }
  }
}

function initGetAsset(assets: any) {
  return async function getAsset(name: string) {
    const asset = assets.find((a: any) => a.name == name)

    console.log('Getting asset...')
    console.log(name)
    console.log(asset)

    if (asset == undefined) return null

    if (name.endsWith('.png')) {
      const blob = new Blob([asset.content])
      const url = URL.createObjectURL(blob)
      const img = new Image()
      img.src = url

      await new Promise(res => {
        img.addEventListener('load', res, false)
      })

      return img
    }

    return asset.content
  }
}

function relative(pos: Vector) {
  return new Vector(pos.x * 1920, pos.y * 1080, pos.z, pos.w)
}

function absolute(pos: Vector) {
  return new Vector(pos.x / 1920, pos.y / 1080, pos.z, pos.w)
}

function lerp(a: number, b: number, t: number) {
  return a + (b - a) * t
}
