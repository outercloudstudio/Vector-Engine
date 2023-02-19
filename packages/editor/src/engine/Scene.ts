import { Engine } from '@/engine/Engine'
import { uuid } from '@/engine/Math'
import { Element } from '@/engine/Element'
import { isGenerator } from '@/engine/Utils'
import { Vector } from '@/engine/Vector'
import {
  Rect,
  Ellipse,
  Image as ImageBuilder,
  Text,
  Link,
  Builder,
  TransformBuilder,
  RenderingBuilder,
} from '@/engine/Builders'

export function useSceneContext(scene: Scene) {
  return {
    Vector,

    Builders: {
      Rect,
      Ellipse,
      Image: ImageBuilder,
      Link,
      Text,
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
            const canvas = new OffscreenCanvas(1920, 1080)
            const ctx: OffscreenCanvasRenderingContext2D = <
              OffscreenCanvasRenderingContext2D
            >canvas.getContext('2d')

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

    ElementTransitions: {
      Fade: function (inputCanvas: OffscreenCanvas, element: Element) {
        const bounds = (<RenderingBuilder>element.builder).bounds()
        const canvas = new OffscreenCanvas(bounds.x, bounds.y)
        const ctx: OffscreenCanvasRenderingContext2D = <
          OffscreenCanvasRenderingContext2D
        >canvas.getContext('2d')

        ctx.globalAlpha = element.transitionProgress

        ctx.drawImage(inputCanvas, 0, 0)

        return canvas
      },

      Circle: function (inputCanvas: OffscreenCanvas, element: Element) {
        const bounds = (<RenderingBuilder>element.builder).bounds()
        const canvas = new OffscreenCanvas(bounds.x, bounds.y)
        const ctx: OffscreenCanvasRenderingContext2D = <
          OffscreenCanvasRenderingContext2D
        >canvas.getContext('2d')

        const targetSize = Math.sqrt(
          Math.pow(bounds.x, 2) + Math.pow(bounds.y, 2)
        )

        ctx.beginPath()
        ctx.ellipse(
          bounds.x / 2,
          bounds.y / 2,
          (element.transitionProgress * targetSize) / 2,
          (element.transitionProgress * targetSize) / 2,
          0,
          0,
          Math.PI * 2
        )
        ctx.closePath()
        ctx.clip()
        ctx.drawImage(inputCanvas, 0, 0)

        return canvas
      },

      CircleCut: function (inputCanvas: OffscreenCanvas, element: Element) {
        const bounds = (<RenderingBuilder>element.builder).bounds()
        const canvas = new OffscreenCanvas(bounds.x, bounds.y)
        const ctx: OffscreenCanvasRenderingContext2D = <
          OffscreenCanvasRenderingContext2D
        >canvas.getContext('2d')

        const targetSize = Math.sqrt(
          Math.pow(bounds.x, 2) + Math.pow(bounds.y, 2)
        )

        ctx.beginPath()
        ctx.rect(0, 0, bounds.x, bounds.y)
        ctx.ellipse(
          bounds.x / 2,
          bounds.y / 2,
          (element.transitionProgress * targetSize) / 2,
          (element.transitionProgress * targetSize) / 2,
          0,
          0,
          Math.PI * 2
        )
        ctx.closePath()
        ctx.clip('evenodd')
        ctx.drawImage(inputCanvas, 0, 0)

        return canvas
      },
    },

    Element,

    defineName(name: string) {
      scene.name = name
    },

    createElement(builder: typeof Builder, options: object) {
      const element = new Element(scene, builder, options)

      scene.addElement(element)

      return element
    },

    removeElement(element: Element) {
      for (
        let elementIndex = 0;
        elementIndex < scene.elements.length;
        elementIndex++
      ) {
        if (scene.elements[elementIndex].id == element.id) {
          scene.elements.splice(elementIndex, 1)

          break
        }
      }

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

    waitForMarker: function* (name: string, offset?: number) {
      while (
        !scene.engine.markers.find(
          marker =>
            marker.name == name &&
            marker.frame ==
              scene.engine.frame -
                Math.ceil((offset || 0) * scene.engine.frameRate)
        )
      ) {
        yield null
      }
    },

    waitWhile: async function* (condition: any) {
      while (await condition()) yield null
    },

    waitForTransition: function* () {
      while (scene.engine.scenes[0].id != scene.id) yield null
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

    aside(context: any) {
      context.next()

      if (context.done == 'true') return

      scene.sideContexts.push(context)
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
        // NOTE: may not need to be async
        const context = await contextLambda()

        yield* context
      }
    },

    transition: async function* (sceneContext: any, transition: any) {
      const targetScene = new Scene(sceneContext, scene.engine)

      await targetScene.load()

      yield* transition({
        load() {
          scene.engine.scenes.push(targetScene)
        },
        unload() {
          scene.engine.scenes.splice(
            scene.engine.scenes.findIndex(s => s.id == scene.id),
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

export class Scene {
  name: string = 'Scene'

  context: any
  unloadedContext: any
  loaded: boolean = false

  engine: Engine
  elements: Element[] = []
  sideContexts: any[] = []
  transitionRenderModifier: any = null
  id: string = uuid()

  constructor(context: any, engine: Engine) {
    this.unloadedContext = context
    this.engine = engine
  }

  async load() {
    try {
      this.context = await this.unloadedContext(useSceneContext(this))
    } catch (error) {
      console.error(error)

      if (this.engine.onError) this.engine.onError(<string>error)

      return
    }

    this.loaded = true

    await this.next()
  }

  async render() {
    const start = Date.now()

    const canvas = new OffscreenCanvas(1920, 1080)
    const ctx: OffscreenCanvasRenderingContext2D = <
      OffscreenCanvasRenderingContext2D
    >canvas.getContext('2d')

    ctx.fillStyle = '#ffffff'
    ctx.fillRect(0, 0, 1920, 1080)

    let sortedElements = this.elements.sort((a: any, b: any) => {
      return (a.priority || 0) - (b.priority || 0)
    })

    try {
      for (const element of sortedElements) {
        if (!element.isRendering) continue

        ctx.translate(0, 1080)
        ctx.scale(1, -1)

        await element.render(ctx)

        ctx.resetTransform()
      }

      if (this.transitionRenderModifier)
        return this.transitionRenderModifier(canvas)
    } catch (error) {
      if (this.engine.onError) this.engine.onError(<string>error)
    }

    const end = Date.now()
    const time = end - start

    // console.log(`RENDER: ${time}ms elemsts: ${sortedElements.length}`)

    return canvas
  }

  async next() {
    if (this.context == undefined) return

    const start = Date.now()

    try {
      await Promise.all(
        this.sideContexts.map(
          (context, index) =>
            new Promise<void>(async res => {
              const result = await context.next()

              if (result.done) this.sideContexts[index] = undefined

              res()
            })
        )
      )

      const sides = this.sideContexts.length

      this.sideContexts = this.sideContexts.filter(
        context => context != undefined
      )

      const additionalEnd = Date.now()
      const additionalTime = additionalEnd - start

      let mains = 1

      let result = (await this.context.next()).value

      while (result != null && result != undefined && isGenerator(result)) {
        mains++

        await result.next()

        if (result.done != 'true') this.sideContexts.push(result)

        result = (await this.context.next()).value
      }

      const mainEnd = Date.now()
      const mainTime = mainEnd - additionalEnd
      const totalTime = mainEnd - start

      // console.log(
      //   `NEXT total: ${totalTime}ms main: ${mainTime}ms side: ${additionalTime}ms sides: ${sides} mains: ${mains}`
      // )
    } catch (error) {
      if (this.engine.onError) this.engine.onError(<string>error)
    }
  }

  addElement(element: Element) {
    this.elements.push(element)
  }
}
