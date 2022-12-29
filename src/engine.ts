import { Vector } from '@/engine-core'

function initAddElement(scene: Scene) {
  return (element: Element) => {
    scene.addElement(element)
  }
}

function initAnimate(frameRate: number) {
  return function* (length: number, mode: any, operator: any) {
    for (let i = 1; i <= length * frameRate; i++) {
      operator(mode(i / (length * frameRate)))

      yield null
    }
  }
}

function initAnimateVector(frameRate: number) {
  return function* (a: Vector, b: Vector, length: number, mode: any, operator: any) {
    for (let i = 1; i <= length * frameRate; i++) {
      operator(a.lerp(b, mode(i / (length * frameRate))))

      yield null
    }
  }
}

function initAnimateNumber(frameRate: number) {
  return function* (a: number, b: number, length: number, mode: any, operator: any) {
    for (let i = 1; i <= length * frameRate; i++) {
      operator(lerp(a, b, mode(i / (length * frameRate))))

      yield null
    }
  }
}

function initAnimateVectorProperty(frameRate: number) {
  return function* (obj: any, property: string, a: Vector, b: Vector, length: number, mode: any) {
    for (let i = 1; i <= length * frameRate; i++) {
      obj[property] = a.lerp(b, mode(i / (length * frameRate)))

      yield null
    }
  }
}

function initAnimateNumberProperty(frameRate: number) {
  return function* (obj: any, property: string, a: number, b: number, length: number, mode: any) {
    for (let i = 1; i <= length * frameRate; i++) {
      obj[property] = lerp(a, b, mode(i / (length * frameRate)))

      yield null
    }
  }
}

function initTransitionTo(frameRate: number, scene: Scene, engine: Engine) {
  return function (sceneName: string, length: number, mode: any, transition: any) {
    const targetScene = engine.scenes.find(s => s.name == sceneName)

    if (targetScene == undefined) {
      console.warn(`Could not find target scene '${sceneName}' to transtiion to!`)

      return
    }

    if (engine.transitionScenes.find(tran => tran.scene == scene) != undefined) {
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

const Modes = {
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
}

const Builders = {
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

        ctx.translate(me.position.x + me.size.x / 2, me.position.y + me.size.y / 2)
        ctx.rotate((me.rotation * Math.PI) / 180)

        ctx.translate(-me.position.x + -me.size.x / 2, -me.position.y + -me.size.y / 2)

        ctx.fillStyle = `rgba(${red},${blue},${green},${alpha})`
        ctx.fillRect(me.position.x - me.size.x * me.origin.x, me.position.y - me.size.y * me.origin.y, me.size.x, me.size.y)
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
        ctx.arc(me.position.x - me.size * 2 * (me.origin.x - 0.5), me.position.y - me.size * 2 * (me.origin.y - 0.5), me.size, 0, 2 * Math.PI)
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

        ctx.translate(me.position.x + me.size.x / 2, me.position.y + me.size.y / 2)
        ctx.rotate((me.rotation * Math.PI) / 180)

        ctx.translate(-me.position.x + -me.size.x / 2, -me.position.y + -me.size.y / 2)

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

function initWaitForMarker(engine: Engine, markers: { name: string; frame: number }[]) {
  return function* (name: string) {
    const targetMarker = markers.find(marker => marker.name == name)

    while (targetMarker == undefined || targetMarker.frame - 1 != engine.currentFrame) {
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

class Scene {
  context: any

  name: string = 'ERROR SCENE NAME NOT SET'

  extraContexts: any[] = []

  elements: Element[] = []

  ctx: any

  constructor(code: string, name: string, frameRate: number, engine: Engine) {
    const codeValue = `return async function*(Vector, Element, Mode, Builder, addElement, animate, all, any, wait, aside, forever, waitWhile, transitionTo, Transition, waitForMarker, getAsset, relative, absolute, lerp, animateVector, animateNumber, animateVectorProperty, animateNumberProperty) { ${code} }`

    this.context = new Function(codeValue)()(
      Vector,
      Element,
      Modes,
      Builders,
      initAddElement(this),
      initAnimate(frameRate),
      all,
      any,
      initWait(frameRate),
      aside,
      forever,
      waitWhile,
      initTransitionTo(frameRate, this, engine),
      Transition,
      initWaitForMarker(engine, engine.markers),
      initGetAsset(engine.assets),
      relative,
      absolute,
      lerp,
      initAnimateVector(frameRate),
      initAnimateNumber(frameRate),
      initAnimateVectorProperty(frameRate),
      initAnimateNumberProperty(frameRate)
    )

    const canvas = document.createElement('canvas')
    canvas.width = 1920
    canvas.height = 1080

    this.ctx = canvas.getContext('2d')

    this.name = name
  }

  addElement(element: Element) {
    this.elements.push(element)
  }

  async render() {
    this.ctx.fillStyle = '#ffffff'
    this.ctx.fillRect(0, 0, 1920, 1080)

    let sortedElements = this.elements.sort((a: any, b: any) => {
      return (a.priority || 0) - (b.priority || 0)
    })

    for (const element of sortedElements) {
      this.ctx.translate(0, 1080)
      this.ctx.scale(1, -1)

      await element.render(this.ctx)

      this.ctx.resetTransform()
    }
  }

  async next() {
    for (let i = 0; i < this.extraContexts.length; i++) {
      const generator = this.extraContexts[i]

      await generator.next()

      if (generator.done == 'true') {
        this.extraContexts.splice(i, 1)

        i--
      }
    }

    let result = (await this.context.next()).value

    while (result != null && result != undefined && typeof result[Symbol.iterator] === 'function') {
      this.extraContexts.push(result)

      result = (await this.context.next()).value
    }
  }
}

export class Engine {
  activeScene: Scene | undefined = undefined

  transitionScenes: {
    scene: Scene
    transition: any
    leftFrames: number
    length: number
    mode: any
  }[] = []

  scenes: {
    name: string
    content: string
  }[] = []

  markers: {
    name: string
    frame: number
  }[] = []

  code: string = ''
  name: string = ''
  frameRate: number = 60

  currentFrame: number = -1

  assets: any

  async render(ctx: any) {
    if (this.activeScene == undefined) return

    await this.activeScene.render()

    ctx.drawImage(this.activeScene.ctx.canvas, 0, 0)

    for (const transitionScene of this.transitionScenes) {
      await transitionScene.scene.render()

      transitionScene.transition(transitionScene.scene.ctx, ctx, transitionScene.mode(transitionScene.leftFrames / transitionScene.length))
    }
  }

  reloadContext(
    code: string,
    name: string,
    frameRate: number,
    scenes: { name: string; content: string }[],
    markers: { name: string; frame: number }[],
    assets: any
  ) {
    this.assets = assets

    this.scenes = scenes

    this.markers = markers

    this.code = code
    this.name = name
    this.frameRate = frameRate

    this.currentFrame = -1

    this.activeScene = new Scene(code, name, frameRate, this)
  }

  async stepFrame() {
    if (this.activeScene == undefined) return

    try {
      await this.activeScene.next()

      for (let i = 0; i < this.transitionScenes.length; i++) {
        const transitionScene = this.transitionScenes[i]
        await transitionScene.scene.next()

        transitionScene.leftFrames--

        if (transitionScene.leftFrames <= 0) {
          this.activeScene = transitionScene.scene

          this.transitionScenes.splice(i, 1)

          i--
        }
      }
    } catch (err) {
      console.error(err)
    }

    this.currentFrame++
  }

  async inference(length: number) {
    const iEngine = new Engine()

    iEngine.reloadContext(this.code, this.name, this.frameRate, this.scenes, this.markers, this.assets)

    let snapshots: {
      frame: number
      name: string
      kind: string
      fakeLength?: number
    }[] = []

    let lastSnapshotScene = null

    let pastTransitions: {
      scene: Scene
      length: number
    }[] = []

    for (let i = 0; i < length; i++) {
      await iEngine.stepFrame()

      if (iEngine.activeScene != lastSnapshotScene) {
        lastSnapshotScene = iEngine.activeScene

        snapshots.push({
          frame: i,
          name: iEngine.activeScene!.name,
          kind: 'start',
        })
      }

      for (const transition of iEngine.transitionScenes) {
        if (pastTransitions.find(t => t.scene == transition.scene) != undefined) continue

        snapshots.push({
          frame: i,
          name: transition.scene.name,
          kind: 'transition',
          fakeLength: transition.length,
        })

        pastTransitions.push({
          scene: transition.scene,
          length: transition.length,
        })
      }

      for (let j = 0; j < pastTransitions.length; j++) {
        if (iEngine.transitionScenes.find(t => t.scene == pastTransitions[j].scene) != undefined) continue

        pastTransitions.splice(j, 1)

        j--
      }
    }

    let events = []

    for (let i = 0; i < snapshots.length; i++) {
      const snapshot = snapshots[i]
      const prevSnapshot = snapshots[i - 1]
      const nextSnapshot = snapshots[i + 1]

      let shouldShowName = true

      if (
        prevSnapshot &&
        prevSnapshot.kind == 'transition' &&
        snapshot.kind == 'start' &&
        prevSnapshot.name == snapshot.name &&
        prevSnapshot.fakeLength &&
        prevSnapshot.fakeLength - 1 == snapshot.frame - prevSnapshot.frame
      )
        shouldShowName = false

      if (snapshot.kind == 'start') {
        events.push({
          from: snapshot.frame,
          to: nextSnapshot ? nextSnapshot.frame : length,
          kind: 'scene',
          name: shouldShowName ? snapshot.name : null,
        })
      } else {
        events.push({
          from: snapshot.frame,
          to: nextSnapshot ? nextSnapshot.frame : length,
          kind: 'transition',
          name: shouldShowName ? snapshot.name : null,
          fakeTo: snapshot.fakeLength,
        })
      }
    }

    return events
  }
}
