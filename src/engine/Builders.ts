import { ReactiveVector, Vector } from '@/engine/Vector'
import { Element } from '@/engine/Element'
import { lerp } from '@/engine/Math'

export class Builder {
  element: Element

  constructor(me: Element) {
    this.element = me
  }

  setup(options: any) {
    if (options != null) {
      for (const option of Object.keys(options)) {
        this.element[option] = options[option]
      }
    }
  }

  protected defineAnimatedVectorSetter(property: string) {
    const me = this

    return async function* (vector: Vector, length: number, mode: any) {
      const oldVectorClone = new Vector(
        me.element[property].x,
        me.element[property].y,
        me.element[property].z,
        me.element[property].w
      )

      const newVectorClone = new Vector(vector.x, vector.y, vector.z, vector.w)

      for (
        let i = 1;
        i <= Math.ceil(length * me.element.scene.engine.frameRate);
        i++
      ) {
        me.element[property] = oldVectorClone.lerp(
          newVectorClone,
          mode(i / Math.ceil(length * me.element.scene.engine.frameRate))
        )

        yield null
      }
    }
  }

  protected defineAnimatedNumberSetter(property: string) {
    const me = this

    return async function* (value: number, length: number, mode: any) {
      const oldValue = me.element[property]

      for (
        let i = 1;
        i <= Math.ceil(length * me.element.scene.engine.frameRate);
        i++
      ) {
        me.element[property] = lerp(
          oldValue,
          value,
          mode(i / Math.ceil(length * me.element.scene.engine.frameRate))
        )

        yield null
      }
    }
  }
}

export class TransformBuilder extends Builder {
  setup(options: any) {
    this.element.position = new Vector(0, 0)
    this.element.animatePosition = this.defineAnimatedVectorSetter('position')
    this.element.rotation = 0
    this.element.animateRotation = this.defineAnimatedNumberSetter('rotation')
    this.element.scale = new Vector(1, 1)
    this.element.animateScale = this.defineAnimatedVectorSetter('scale')

    super.setup(options)
  }
}

export class Link extends TransformBuilder {
  setup(options: any) {
    this.element.links = []

    super.setup(options)

    this.element._position = new ReactiveVector(
      (position: Vector, oldPosition: Vector) => {
        this.updateLinkedElements(
          position,
          oldPosition,
          this.element.rotation,
          this.element.rotation,
          this.element._scale,
          this.element._scale
        )
      },
      this.element.position.x,
      this.element.position.y,
      this.element.position.z,
      this.element.position.w
    )

    this.element._rotation = this.element.rotation

    this.element._scale = new ReactiveVector(
      (scale: Vector, oldScale: Vector) => {
        this.updateLinkedElements(
          this.element._position,
          this.element._position,
          this.element._rotation,
          this.element._rotation,
          scale,
          oldScale
        )
      },
      this.element.scale.x,
      this.element.scale.y,
      this.element.scale.z,
      this.element.scale.w
    )

    Object.defineProperty(this.element, 'position', {
      get: () => this.element._position,
      set: (position: Vector) => {
        const oldPosition = new Vector(
          this.element._position.x,
          this.element._position.y,
          this.element._position.z,
          this.element._position.w
        )

        this.element._position = new ReactiveVector(
          (position: Vector, oldPosition: Vector) => {
            this.updateLinkedElements(
              position,
              oldPosition,
              this.element._rotation,
              this.element._rotation,
              this.element._scale,
              this.element._scale
            )
          },
          position.x,
          position.y,
          position.z,
          position.w
        )

        this.updateLinkedElements(
          position,
          oldPosition,
          this.element._rotation,
          this.element._rotation,
          this.element._scale,
          this.element._scale
        )
      },
    })

    Object.defineProperty(this.element, 'rotation', {
      get: () => this.element._rotation,
      set: (rotation: number) => {
        const oldRotation = this.element._rotation

        this.element._rotation = rotation

        this.updateLinkedElements(
          this.element._position,
          this.element._position,
          rotation,
          oldRotation,
          this.element._scale,
          this.element._scale
        )
      },
    })

    Object.defineProperty(this.element, 'scale', {
      get: () => this.element._position,
      set: (scale: Vector) => {
        const oldScale = new Vector(
          this.element._scale.x,
          this.element._scale.y,
          this.element._scale.z,
          this.element._scale.w
        )

        this.element._scale = new ReactiveVector(
          (scale: Vector, oldScale: Vector) => {
            this.updateLinkedElements(
              this.element._position,
              this.element._position,
              this.element._rotation,
              this.element._rotation,
              scale,
              oldScale
            )
          },
          scale.x,
          scale.y,
          scale.z,
          scale.w
        )

        this.updateLinkedElements(
          this.element._position,
          this.element._position,
          this.element._rotation,
          this.element._rotation,
          scale,
          oldScale
        )
      },
    })
  }

  updateLinkedElements(
    newPosition: Vector,
    oldPosition: Vector,
    newRotation: number,
    oldRotation: number,
    newScale: Vector,
    oldScale: Vector
  ) {
    const positionDelta = newPosition.subtract(oldPosition)
    const rotationDelta = newRotation - oldRotation
    const rotationDeltaRad = (rotationDelta / 180) * Math.PI
    const scaleDelta = newScale.divide(oldScale)

    for (const element of this.element.links) {
      if (!(element.builder instanceof TransformBuilder))
        throw new Error('Link can not update a non transform buffer')

      element.position = element.position.add(positionDelta)
      element.rotation = element.rotation + rotationDelta

      let elementPositionDelta = element.position.subtract(newPosition)

      const rotatedElementPositionDeltaX =
        Math.cos(rotationDeltaRad) * elementPositionDelta.x -
        Math.sin(rotationDeltaRad) * elementPositionDelta.y

      const rotatedElementPositionDeltaY =
        Math.cos(rotationDeltaRad) * elementPositionDelta.y +
        Math.sin(rotationDeltaRad) * elementPositionDelta.x

      element.position = new Vector(
        newPosition.x + rotatedElementPositionDeltaX,
        newPosition.y + rotatedElementPositionDeltaY,
        element.position.z,
        element.position.w
      )

      elementPositionDelta = element.position.subtract(newPosition)

      const scaledPositionDelta = elementPositionDelta.multiply(scaleDelta)

      element.position = newPosition.add(scaledPositionDelta)
      element.scale = element.scale.multiply(scaleDelta)
    }
  }
}

export class RenderingBuilder extends TransformBuilder {
  setup(options: any) {
    this.element.origin = new Vector(0.5, 0.5)
    this.element.animateOrigin = this.defineAnimatedVectorSetter('origin')
    this.element.priority = 0
    this.element.isRendering = true
    this.element.renderingModifier = undefined
    this.element.transitionProgress = 0
    this.element.transition = this.defineTransition()
    this.element.transitionOut = this.defineTransitionOut()

    super.setup(options)
  }

  bounds(): Vector {
    throw new Error('bounds() is not implemented on this builder!')
  }

  extent(): Vector {
    throw new Error('extent() is not implemented on this builder!')
  }

  extentOffset(): Vector {
    throw new Error('extentOffset() is not implemented on this builder!')
  }

  async render(ctx: CanvasRenderingContext2D) {
    throw new Error('render() is not implemented on this builder!')
  }

  defineTransition() {
    const me = this

    return async function* (
      time: number,
      transition: (
        canvas: HTMLCanvasElement,
        element: Element
      ) => HTMLCanvasElement
    ) {
      me.element.renderingModifier = transition

      for (
        let i = 1;
        i <= Math.ceil(time * me.element.scene.engine.frameRate);
        i++
      ) {
        me.element.transitionProgress =
          i / Math.ceil(time * me.element.scene.engine.frameRate)

        yield null
      }

      me.element.renderingModifier = undefined
    }
  }

  defineTransitionOut() {
    const me = this

    return async function* (
      time: number,
      transition: (
        canvas: HTMLCanvasElement,
        element: Element
      ) => HTMLCanvasElement
    ) {
      me.element.renderingModifier = transition

      for (
        let i = 1;
        i <= Math.ceil(time * me.element.scene.engine.frameRate);
        i++
      ) {
        me.element.transitionProgress =
          1 - i / Math.ceil(time * me.element.scene.engine.frameRate)

        yield null
      }

      me.element.renderingModifier = undefined
    }
  }
}

export class Rect extends RenderingBuilder {
  setup(options: any) {
    this.element.size = new Vector(100, 100)
    this.element.animateSize = this.defineAnimatedVectorSetter('size')
    this.element.color = new Vector(0, 0, 0, 1)
    this.element.animateColor = this.defineAnimatedVectorSetter('color')
    this.element.outlineColor = new Vector(0, 0, 0, 0)
    this.element.animateOutlineColor =
      this.defineAnimatedVectorSetter('outlineColor')
    this.element.outlineWidth = 0
    this.element.animateOutlineWidth =
      this.defineAnimatedNumberSetter('outlineWidth')
    this.element.radius = 0
    this.element.animateRadius = this.defineAnimatedNumberSetter('radius')

    super.setup(options)
  }

  bounds() {
    return new Vector(
      this.element.size.x * this.element.scale.x + this.element.outlineWidth,
      this.element.size.y * this.element.scale.y + this.element.outlineWidth
    )
  }

  extent() {
    return new Vector(
      this.element.size.x * this.element.scale.x,
      this.element.size.y * this.element.scale.y
    )
  }

  extentOffset() {
    return new Vector(
      this.element.outlineWidth / 2,
      this.element.outlineWidth / 2
    )
  }

  async render(ctx: CanvasRenderingContext2D) {
    const red = this.element.color.x * 255
    const blue = this.element.color.y * 255
    const green = this.element.color.z * 255
    const alpha = this.element.color.w

    const outlineRed = this.element.outlineColor.x * 255
    const outlineBlue = this.element.outlineColor.y * 255
    const outlineGreen = this.element.outlineColor.z * 255
    const outlineAlpha = this.element.outlineColor.w

    ctx.fillStyle = `rgba(${red},${blue},${green},${alpha})`
    ctx.strokeStyle = `rgba(${outlineRed},${outlineBlue},${outlineGreen},${outlineAlpha})`
    ctx.lineWidth = this.element.outlineWidth

    ctx.beginPath()

    ctx.roundRect(
      this.element.outlineWidth / 2,
      this.element.outlineWidth / 2,
      this.element.size.x * this.element.scale.x,
      this.element.size.y * this.element.scale.y,
      this.element.radius
    )

    ctx.fill()
    if (this.element.outlineWidth != 0) ctx.stroke()
  }
}

export class Ellipse extends RenderingBuilder {
  setup(options: any) {
    this.element.size = new Vector(100, 100)
    this.element.animateSize = this.defineAnimatedVectorSetter('size')
    this.element.color = new Vector(0, 0, 0, 1)
    this.element.animateColor = this.defineAnimatedVectorSetter('color')
    this.element.outlineColor = new Vector(0, 0, 0, 0)
    this.element.animateOutlineColor =
      this.defineAnimatedVectorSetter('outlineColor')
    this.element.outlineWidth = 0
    this.element.animateOutlineWidth =
      this.defineAnimatedNumberSetter('outlineWidth')

    super.setup(options)
  }

  bounds() {
    return new Vector(
      this.element.size.x * this.element.scale.x + this.element.outlineWidth,
      this.element.size.y * this.element.scale.y + this.element.outlineWidth
    )
  }

  extent() {
    return new Vector(
      this.element.size.x * this.element.scale.x,
      this.element.size.y * this.element.scale.y
    )
  }

  extentOffset() {
    return new Vector(
      this.element.outlineWidth / 2,
      this.element.outlineWidth / 2
    )
  }

  async render(ctx: CanvasRenderingContext2D) {
    const red = this.element.color.x * 255
    const blue = this.element.color.y * 255
    const green = this.element.color.z * 255
    const alpha = this.element.color.w

    const outlineRed = this.element.outlineColor.x * 255
    const outlineBlue = this.element.outlineColor.y * 255
    const outlineGreen = this.element.outlineColor.z * 255
    const outlineAlpha = this.element.outlineColor.w

    ctx.fillStyle = `rgba(${red},${blue},${green},${alpha})`
    ctx.strokeStyle = `rgba(${outlineRed},${outlineBlue},${outlineGreen},${outlineAlpha})`
    ctx.lineWidth = this.element.outlineWidth

    ctx.beginPath()

    ctx.ellipse(
      (this.element.size.x / 2) * this.element.scale.x +
        this.element.outlineWidth / 2,
      (this.element.size.y / 2) * this.element.scale.y +
        this.element.outlineWidth / 2,
      (this.element.size.x / 2) * this.element.scale.x,
      (this.element.size.y / 2) * this.element.scale.y,
      0,
      0,
      2 * Math.PI
    )

    ctx.fill()
    if (this.element.outlineWidth != 0) ctx.stroke()
  }
}

export const Image = {
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
}
