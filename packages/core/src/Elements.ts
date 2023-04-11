import { Scene } from './Scene'
import { lerp, uuid } from './Math'
import { Vector } from './Vector'
import { color } from './Color'

export class Element {
  scene: Scene | undefined
  id: string = uuid()

  constructor(options: {
    [Property in keyof Element]?: Element[Property]
  }) {
    if (options != null) {
      for (const option of Object.keys(options)) {
        this[option] = options[option]
      }
    }
  }
}

export class TransformElement extends Element {
  position: Vector = new Vector(0, 0, 0, 0)
  rotation: number = 0
  scale: Vector = new Vector(1, 1)

  async *animatePosition(vector: Vector, length: number, mode: any) {
    if (this.scene == undefined)
      throw new Error('Can not animate without being added to a scene!')

    const oldVectorClone = new Vector(
      this.position.x,
      this.position.y,
      this.position.z,
      this.position.w
    )

    const newVectorClone = new Vector(vector.x, vector.y, vector.z, vector.w)

    for (let i = 1; i <= Math.ceil(length * this.scene.engine.frameRate); i++) {
      this.position = oldVectorClone.lerp(
        newVectorClone,
        mode(i / Math.ceil(length * this.scene.engine.frameRate))
      )

      yield null
    }
  }

  async *animateRotation(value: number, length: number, mode: any) {
    if (this.scene == undefined)
      throw new Error('Can not animate without being added to a scene!')

    const oldValue = this.rotation

    for (let i = 1; i <= Math.ceil(length * this.scene.engine.frameRate); i++) {
      this.rotation = lerp(
        oldValue,
        value,
        mode(i / Math.ceil(length * this.scene.engine.frameRate))
      )

      yield null
    }
  }

  async *animateScale(vector: Vector, length: number, mode: any) {
    if (this.scene == undefined)
      throw new Error('Can not animate without being added to a scene!')

    const oldVectorClone = new Vector(
      this.scale.x,
      this.scale.y,
      this.scale.z,
      this.scale.w
    )

    const newVectorClone = new Vector(vector.x, vector.y, vector.z, vector.w)

    for (let i = 1; i <= Math.ceil(length * this.scene.engine.frameRate); i++) {
      this.scale = oldVectorClone.lerp(
        newVectorClone,
        mode(i / Math.ceil(length * this.scene.engine.frameRate))
      )

      yield null
    }
  }
}

export class RenderElement extends TransformElement {
  origin: Vector = new Vector(0.5, 0.5)

  async render(canvas: OffscreenCanvas) {
    throw new Error('Render method not implemented.')
  }

  async *animateOrigin(vector: Vector, length: number, mode: any) {
    if (this.scene == undefined)
      throw new Error('Can not animate without being added to a scene!')

    const oldVectorClone = new Vector(
      this.origin.x,
      this.origin.y,
      this.origin.z,
      this.origin.w
    )

    const newVectorClone = new Vector(vector.x, vector.y, vector.z, vector.w)

    for (let i = 1; i <= Math.ceil(length * this.scene.engine.frameRate); i++) {
      this.origin = oldVectorClone.lerp(
        newVectorClone,
        mode(i / Math.ceil(length * this.scene.engine.frameRate))
      )

      yield null
    }
  }
}

export class Rect extends RenderElement {
  size: Vector = new Vector(100, 100)
  color: Vector = color('#000000')
  outline: Vector = color('#000000')
  outlineWidth: number = 0
  radius: number = 0

  constructor(options: {
    [Property in keyof Rect]?: Rect[Property]
  }) {
    super({})

    if (options != null) {
      for (const option of Object.keys(options)) {
        this[option] = options[option]
      }
    }
  }

  async render(canvas: OffscreenCanvas) {
    const ctx = canvas.getContext('2d')

    ctx.translate(this.position.x, this.position.y)
    ctx.rotate((this.rotation * Math.PI) / 180)
    ctx.translate(-this.size.x * this.origin.x, -this.size.y * this.origin.x)

    const red = this.color.x * 255
    const green = this.color.y * 255
    const blue = this.color.z * 255
    const alpha = this.color.w

    const outlineRed = this.outline.x * 255
    const outlineGreen = this.outline.y * 255
    const outlineBlue = this.outline.z * 255
    const outlineAlpha = this.outline.w

    ctx.fillStyle = `rgba(${red},${blue},${green},${alpha})`
    ctx.strokeStyle = `rgba(${outlineRed},${outlineBlue},${outlineGreen},${outlineAlpha})`
    ctx.lineWidth = this.outlineWidth

    const width = this.size.x * this.scale.x
    const height = this.size.y * this.scale.y
    const radiusX = Math.min(Math.max(this.radius, 0) * this.scale.x, width / 2)
    const radiusY = Math.min(
      Math.max(this.radius, 0) * this.scale.y,
      height / 2
    )

    ctx.beginPath()
    ctx.moveTo(Math.min(radiusX, width / 2), 0)
    ctx.lineTo(width - radiusX, 0)
    ctx.quadraticCurveTo(width, 0, width, radiusY)
    ctx.lineTo(width, height - radiusY)
    ctx.quadraticCurveTo(width, height, width - radiusX, height)
    ctx.lineTo(radiusX, height)
    ctx.quadraticCurveTo(0, height, 0, height - radiusY)
    ctx.lineTo(0, radiusY)
    ctx.quadraticCurveTo(0, 0, radiusX, 0)
    ctx.closePath()

    ctx.fill()
    if (this.outlineWidth != 0) ctx.stroke()
  }
}

export class Ellipse extends RenderElement {
  size: Vector = new Vector(100, 100)
  color: Vector = color('#000000')
  outline: Vector = color('#000000')
  outlineWidth: number = 0

  constructor(options: {
    [Property in keyof Ellipse]?: Ellipse[Property]
  }) {
    super({})

    if (options != null) {
      for (const option of Object.keys(options)) {
        this[option] = options[option]
      }
    }
  }

  async render(canvas: OffscreenCanvas) {
    const ctx = canvas.getContext('2d')

    ctx.translate(this.position.x, this.position.y)
    ctx.rotate((this.rotation * Math.PI) / 180)
    ctx.translate(-this.size.x * this.origin.x, -this.size.y * this.origin.x)

    const red = this.color.x * 255
    const green = this.color.y * 255
    const blue = this.color.z * 255
    const alpha = this.color.w

    const outlineRed = this.outline.x * 255
    const outlineGreen = this.outline.y * 255
    const outlineBlue = this.outline.z * 255
    const outlineAlpha = this.outline.w

    ctx.fillStyle = `rgba(${red},${blue},${green},${alpha})`
    ctx.strokeStyle = `rgba(${outlineRed},${outlineBlue},${outlineGreen},${outlineAlpha})`
    ctx.lineWidth = this.outlineWidth

    const width = this.size.x * this.scale.x
    const height = this.size.y * this.scale.y

    ctx.beginPath()
    ctx.ellipse(
      (this.size.x / 2) * this.scale.x + this.outlineWidth / 2,
      (this.size.y / 2) * this.scale.y + this.outlineWidth / 2,
      (this.size.x / 2) * this.scale.x,
      (this.size.y / 2) * this.scale.y,
      0,
      0,
      2 * Math.PI
    )
    ctx.closePath()

    ctx.fill()
    if (this.outlineWidth != 0) ctx.stroke()
  }
}

export class VectorText extends RenderElement {
  text: string = 'Vector Engine'
  size: number = 100
  font: string = 'Mulish'
  color: Vector = color('#000000')
  outline: Vector = color('#000000')
  outlineWidth: number = 0

  constructor(options: {
    [Property in keyof VectorText]?: VectorText[Property]
  }) {
    super({})

    if (options != null) {
      for (const option of Object.keys(options)) {
        this[option] = options[option]
      }
    }
  }

  async render(canvas: OffscreenCanvas) {
    const ctx = canvas.getContext('2d')

    ctx.translate(this.position.x, this.position.y)
    ctx.rotate((this.rotation * Math.PI) / 180)

    const size = this.size * this.scale.x

    ctx.font = `${size}px ${this.font}`

    const measure = ctx.measureText(this.text)

    ctx.translate(
      -measure.width * this.origin.x,
      -measure.actualBoundingBoxAscent * this.origin.y
    )
    ctx.scale(1, -1)

    const red = this.color.x * 255
    const green = this.color.y * 255
    const blue = this.color.z * 255
    const alpha = this.color.w

    const outlineRed = this.outline.x * 255
    const outlineGreen = this.outline.y * 255
    const outlineBlue = this.outline.z * 255
    const outlineAlpha = this.outline.w

    ctx.fillStyle = `rgba(${red},${blue},${green},${alpha})`
    ctx.strokeStyle = `rgba(${outlineRed},${outlineBlue},${outlineGreen},${outlineAlpha})`
    ctx.lineWidth = this.outlineWidth

    ctx.fillText(this.text, 0, 0)
    if (this.outlineWidth != 0) ctx.strokeText(this.text, 0, 0)
  }
}

export class VectorImage extends RenderElement {
  image: HTMLImageElement = undefined
  size: Vector = new Vector(100, 100)
  color: Vector = color('#000000')

  constructor(options: {
    [Property in keyof VectorImage]?: VectorImage[Property]
  }) {
    super({})

    if (options != null) {
      for (const option of Object.keys(options)) {
        this[option] = options[option]
      }
    }
  }

  async render(canvas: OffscreenCanvas) {
    const ctx = canvas.getContext('2d')

    ctx.translate(this.position.x, this.position.y)
    ctx.rotate((this.rotation * Math.PI) / 180)

    const width = this.size.x * this.scale.x
    const height = this.size.y * this.scale.y

    ctx.translate(-width * this.origin.x, -height * this.origin.y)
    ctx.scale(1, -1)

    const imageAspect = this.image.width / this.image.height

    const red = this.color.x * 255
    const green = this.color.y * 255
    const blue = this.color.z * 255
    const alpha = this.color.w

    const bestWidth = Math.max(width, height * imageAspect)
    const bestHeight = bestWidth / imageAspect

    const offsetX = (bestWidth - width) / 2
    const offsetY = (bestHeight - width) / 2

    ctx.imageSmoothingEnabled = false

    const colorCanvas = new OffscreenCanvas(bestHeight, bestHeight)
    const colorCtx = colorCanvas.getContext('2d')

    colorCtx.imageSmoothingEnabled = false

    colorCtx.fillStyle = `rgb(${red}, ${green}, ${blue})`
    colorCtx.fillRect(0, 0, width, height)
    colorCtx.globalCompositeOperation = 'multiply'

    colorCtx.drawImage(this.image, 0, 0, bestWidth, bestHeight)
    colorCtx.globalCompositeOperation = 'destination-atop'

    colorCtx.drawImage(this.image, 0, 0, bestWidth, bestHeight)
    colorCtx.globalCompositeOperation = 'destination-in'

    colorCtx.fillStyle = `rgb(1, 1, 1, ${alpha})`
    colorCtx.fillRect(0, 0, width, height)

    ctx.drawImage(colorCanvas, -offsetX, -offsetY)

    ctx.imageSmoothingEnabled = true
  }
}
