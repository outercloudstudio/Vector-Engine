import { Vector, Element } from '@/engine/EngineCore'

class Builder {
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

class RenderingBuilder extends Builder {
  setup(options: any) {
    this.element.position = new Vector(0, 0)
    this.element.origin = new Vector(0.5, 0.5)
    this.element.rotation = 0
    this.element.size = new Vector(100, 100)
    this.element.priority = 0

    this.element.isRendering = true

    super.setup(options)
  }

  bounds(): Vector {
    throw new Error('bounds() is not implemented on this builder!')
  }

  offset(): Vector {
    throw new Error('offset() is not implemented on this builder!')
  }

  async render(ctx: OffscreenCanvasRenderingContext2D) {
    throw new Error('render() is not implemented on this builder!')
  }
}

export class Rect extends RenderingBuilder {
  setup(options: any) {
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

  offset() {
    return new Vector(
      this.element.outlineWidth / 2,
      this.element.outlineWidth / 2
    )
  }

  async render(ctx: any) {
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

  offset() {
    return new Vector(
      this.element.outlineWidth / 2,
      this.element.outlineWidth / 2
    )
  }

  async render(ctx: any) {
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
