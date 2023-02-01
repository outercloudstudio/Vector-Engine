import { Vector } from '@/engine/Vector'
import { Element } from '@/engine/Element'

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
}

export class TransformBuilder extends Builder {
  setup(options: any) {
    this.element.position = new Vector(0, 0)
    this.element.rotation = 0

    super.setup(options)
  }
}

export class Link extends TransformBuilder {
  setup(options: any) {
    this.element.links = []

    super.setup(options)

    this.element._position = this.element.position
    this.element._rotation = this.element.rotation

    Object.defineProperty(this.element, 'position', {
      get: () => this.element._position,
      set: (position: Vector) =>
        this.updateLinkedElements(position, this.element._rotation),
    })

    Object.defineProperty(this.element, 'rotation', {
      get: () => this.element._rotation,
      set: (rotation: number) =>
        this.updateLinkedElements(this.element._position, rotation),
    })
  }

  updateLinkedElements(newPosition: Vector, newRotation: number) {
    const positionDelta = newPosition.subtract(this.element._position)
    const rotationDelta = newRotation - this.element._rotation
    const rotationDeltaRad = (rotationDelta / 180) * Math.PI

    for (const element of this.element.links) {
      if (!(element.builder instanceof TransformBuilder))
        throw new Error('Link can not update a non transform buffer')

      element.position = element.position.add(positionDelta)
      element.rotation = element.rotation + rotationDelta

      const elementPositionDelta = element.position.subtract(newPosition)

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
    }

    this.element._position = newPosition
    this.element._rotation = newRotation
  }
}

export class RenderingBuilder extends TransformBuilder {
  setup(options: any) {
    this.element.origin = new Vector(0.5, 0.5)
    this.element.priority = 0
    this.element.isRendering = true

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
}

export class Rect extends RenderingBuilder {
  setup(options: any) {
    this.element.size = new Vector(100, 100)
    this.element.color = new Vector(0, 0, 0, 1)
    this.element.outlineColor = new Vector(0, 0, 0, 0)
    this.element.outlineWidth = 0
    this.element.radius = 0

    super.setup(options)
  }

  bounds() {
    return new Vector(
      this.element.size.x + this.element.outlineWidth,
      this.element.size.y + this.element.outlineWidth
    )
  }

  extent() {
    return new Vector(this.element.size.x, this.element.size.y)
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
      this.element.size.x,
      this.element.size.y,
      this.element.radius
    )

    ctx.fill()
    if (this.element.outlineWidth != 0) ctx.stroke()
  }
}

export class Ellipse extends RenderingBuilder {
  setup(options: any) {
    this.element.size = new Vector(100, 100)
    this.element.color = new Vector(0, 0, 0, 1)
    this.element.outlineColor = new Vector(0, 0, 0, 0)
    this.element.outlineWidth = 0

    super.setup(options)
  }

  bounds() {
    return new Vector(
      this.element.size.x + this.element.outlineWidth,
      this.element.size.y + this.element.outlineWidth
    )
  }

  extent() {
    return new Vector(this.element.size.x, this.element.size.y)
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
      this.element.size.x / 2 + this.element.outlineWidth / 2,
      this.element.size.y / 2 + this.element.outlineWidth / 2,
      this.element.size.x / 2,
      this.element.size.y / 2,
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
