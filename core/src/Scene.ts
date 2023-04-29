import { Engine } from './Engine'
import { uuid } from './Math'
import { Element, RenderElement } from './Elements'
import { Vector } from './Vector'
import { Aside } from './Aside'

export type SceneContext = {
  Transitions: {
    Fade(render: CanvasImageSource, time: number): OffscreenCanvas
  }
  add<E extends Element>(element: E): E
  remove<E extends Element>(element: E): E
  animate(length: number, mode: any, operator: any): AsyncGenerator
  animateVector(
    property: any,
    a: Vector,
    b: Vector,
    length: number,
    mode: any
  ): Generator
  wait(length: number): Generator
  waitForMarker(name: string, offset?: number): Generator
  relative(position: Vector): Vector
  absolute(position: Vector): Vector
  seconds(frames: number): number
  minutesFrames(frames: number): number
  frames(seconds: number): number
  aside(
    context:
      | (() => Generator)
      | (() => AsyncGenerator)
      | AsyncGenerator
      | Generator
  ): Promise<Aside>
}

export function useSceneContext(scene: Scene): SceneContext {
  return {
    Transitions: {
      Fade: (render: CanvasImageSource, time: number) => {
        const canvas = new OffscreenCanvas(1920, 1080)
        const ctx: OffscreenCanvasRenderingContext2D = <
          OffscreenCanvasRenderingContext2D
        >canvas.getContext('2d')

        ctx.globalAlpha = time
        ctx.drawImage(render, 0, 0)

        return canvas
      },
    },

    add<E extends Element>(element: E): E {
      element.scene = scene

      scene.addElement(element)

      return element
    },

    remove<E extends Element>(element: E): E {
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

    relative(position: Vector) {
      return new Vector(
        position.x * 1920,
        position.y * 1080,
        position.z,
        position.w
      )
    },

    absolute(position: Vector) {
      return new Vector(
        position.x / 1920,
        position.y / 1080,
        position.z,
        position.w
      )
    },

    seconds(seconds: number) {
      return Math.ceil(seconds * scene.engine.frameRate)
    },

    minutesFrames(minutes: number) {
      return Math.ceil(minutes * 60 * scene.engine.frameRate)
    },

    frames(frames: number) {
      return frames / scene.engine.frameRate
    },

    async aside(
      context:
        | (() => Generator)
        | (() => AsyncGenerator)
        | AsyncGenerator
        | Generator
    ): Promise<Aside> {
      const aside = new Aside(context)
      await aside.next()

      if (aside.done) return

      scene.asides.push(aside)

      return aside
    },
  }
}

export class Scene {
  context: any
  unloadedContext: any
  loaded: boolean = false

  engine: Engine
  elements: Element[] = []
  asides: Aside[] = []

  transition: any = null
  transitionLength: number = 0

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
    const canvas = new OffscreenCanvas(1920, 1080)
    const ctx: OffscreenCanvasRenderingContext2D = <
      OffscreenCanvasRenderingContext2D
    >canvas.getContext('2d')

    let sortedElements = this.elements.sort((a: any, b: any) => {
      return a.priority - b.priority
    })

    try {
      for (const element of sortedElements) {
        if (!(element instanceof RenderElement)) continue

        ctx.translate(0, 1080)
        ctx.scale(1, -1)

        await element.render(canvas)

        ctx.resetTransform()
      }
    } catch (error) {
      if (this.engine.onError) this.engine.onError(<string>error)
    }
    return canvas
  }

  async next() {
    if (this.context == undefined) {
      console.warn('Undefined context!')

      return
    }

    try {
      await Promise.all(
        this.asides.map(
          (aside, index) =>
            new Promise<void>(async res => {
              if (aside === undefined) {
                res()

                return
              }

              await aside.next()

              if (aside.done) this.asides[index] = undefined

              res()
            })
        )
      )

      this.asides = this.asides.filter(context => context != undefined)

      await this.context.next()
    } catch (error) {
      if (this.engine.onError) this.engine.onError(<string>error)
    }
  }

  addElement(element: Element) {
    this.elements.push(element)
  }
}
