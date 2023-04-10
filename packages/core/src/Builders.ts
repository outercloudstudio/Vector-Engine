import { Vector } from './Vector'
import { Element } from './Element'
import { lerp } from './Math'

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

    this.element._position = this.element.position
    this.element._rotation = this.element.rotation
    this.element._scale = this.element.scale

    Object.defineProperty(this.element, 'position', {
      get: () => this.element._position,
      set: (position: Vector) => {
        this.updateCache()
        this.element._position = position
        this.updatePosition()
      },
    })

    Object.defineProperty(this.element, 'rotation', {
      get: () => this.element._rotation,
      set: (rotation: number) => {
        this.updateCache()
        this.element._rotation = rotation
        this.updateRotation()
      },
    })

    Object.defineProperty(this.element, 'scale', {
      get: () => this.element._scale,
      set: (scale: Vector) => {
        this.updateCache()
        this.element._scale = scale
        this.updateScale()
      },
    })

    this.element._relativeTransforms = {}

    this.updateCache()
  }

  rotateVector(vector: Vector, angle: number) {
    return new Vector(
      vector.x * Math.cos(angle) - vector.y * Math.sin(angle),
      vector.y * Math.cos(angle) + vector.x * Math.sin(angle),
      vector.z,
      vector.w
    )
  }

  updatePosition() {
    for (const link of this.element.links) {
      if (!(link.builder instanceof TransformBuilder))
        throw new Error('Link can not hold a non transform buffer')

      const angle = (this.element.rotation / 180) * Math.PI
      const linkRelativePosition =
        this.element._relativeTransforms[link.id].relativePosition

      link.position = this.element.position.add(
        this.rotateVector(linkRelativePosition, angle).multiply(
          this.element.scale
        )
      )

      this.element._relativeTransforms[link.id].position = link.position
    }
  }

  updateRotation() {
    for (const link of this.element.links) {
      if (!(link.builder instanceof TransformBuilder))
        throw new Error('Link can not hold a non transform buffer')

      link.rotation =
        this.element.rotation +
        this.element._relativeTransforms[link.id].relativeRotation

      this.element._relativeTransforms[link.id].rotation = link.rotation
    }

    this.updatePosition()
  }

  updateScale() {
    for (const link of this.element.links) {
      if (!(link.builder instanceof TransformBuilder))
        throw new Error('Link can not hold a non transform buffer')

      link.scale = this.element.scale.multiply(
        this.element._relativeTransforms[link.id].relativeScale
      )

      this.element._relativeTransforms[link.id].scale = link.scale
    }

    this.updatePosition()
  }

  updateCache() {
    let updatedRelativeTransform: any = {}

    for (const link of this.element.links) {
      if (!(link.builder instanceof TransformBuilder))
        throw new Error('Link can not hold a non transform buffer')

      if (
        this.element._relativeTransforms[link.id] == undefined ||
        !this.element._relativeTransforms[link.id].position.equals(
          link.position
        ) ||
        !this.element._relativeTransforms[link.id].rotation == link.rotation ||
        !this.element._relativeTransforms[link.id].scale.equals(link.scale)
      ) {
        const deltaPosition = link.position.subtract(this.element.position)
        const deltaMagnitude = deltaPosition.magnitude()
        const angle =
          Math.atan2(deltaPosition.y, deltaPosition.x) -
          (this.element.rotation / 180) * Math.PI

        const projectedDeltaPosition = new Vector(
          deltaMagnitude * Math.cos(angle),
          deltaMagnitude * Math.sin(angle),
          deltaPosition.z,
          deltaPosition.w
        )

        const newTransformData = {
          position: link.position.clone(),
          rotation: link.rotation,
          scale: link.scale.clone(),
          relativePosition: projectedDeltaPosition.divide(this.element.scale),
          relativeRotation: link.rotation - this.element.rotation,
          relativeScale: link.scale.divide(this.element.scale),
        }

        updatedRelativeTransform[link.id] = newTransformData
      } else {
        updatedRelativeTransform[link.id] =
          this.element._relativeTransforms[link.id]
      }
    }

    this.element._relativeTransforms = updatedRelativeTransform
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

  async render(ctx: OffscreenCanvasRenderingContext2D) {
    throw new Error('render() is not implemented on this builder!')
  }

  defineTransition() {
    const me = this

    return async function* (
      time: number,
      transition: (
        canvas: OffscreenCanvas,
        element: Element
      ) => OffscreenCanvas,
      mode: any
    ) {
      me.element.renderingModifier = transition

      for (
        let i = 1;
        i <= Math.ceil(time * me.element.scene.engine.frameRate);
        i++
      ) {
        me.element.transitionProgress = mode(
          i / Math.ceil(time * me.element.scene.engine.frameRate)
        )

        yield null
      }

      me.element.renderingModifier = undefined
    }
  }

  defineTransitionOut() {
    const me = this

    return async function* (
      time: number,
      transition: (canvas: OffscreenCanvas, element: Element) => OffscreenCanvas
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
    this.element.directRender = true

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

  async render(ctx: OffscreenCanvasRenderingContext2D) {
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

    const offset = this.element.outlineWidth / 2
    const width = this.element.size.x * this.element.scale.x
    const height = this.element.size.y * this.element.scale.y
    const radiusX = Math.min(
      Math.max(this.element.radius, 0) * this.element.scale.x,
      width / 2
    )
    const radiusY = Math.min(
      Math.max(this.element.radius, 0) * this.element.scale.y,
      height / 2
    )

    ctx.beginPath()
    ctx.moveTo(Math.min(radiusX, width / 2) + offset, offset)
    ctx.lineTo(offset + width - radiusX, offset)
    ctx.quadraticCurveTo(
      offset + width,
      offset,
      offset + width,
      offset + radiusY
    )
    ctx.lineTo(offset + width, offset + height - radiusY)
    ctx.quadraticCurveTo(
      offset + width,
      offset + height,
      offset + width - radiusX,
      offset + height
    )
    ctx.lineTo(offset + radiusX, offset + height)
    ctx.quadraticCurveTo(
      offset,
      offset + height,
      offset,
      offset + height - radiusY
    )
    ctx.lineTo(offset, offset + radiusY)
    ctx.quadraticCurveTo(offset, offset, offset + radiusX, offset)
    ctx.closePath()

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
    this.element.directRender = true

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

  async render(ctx: OffscreenCanvasRenderingContext2D) {
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

export class Image extends RenderingBuilder {
  setup(options: any) {
    this.element.image = undefined
    this.element.color = new Vector(1, 1, 1, 1)
    this.element.animateColor = this.defineAnimatedVectorSetter('color')
    this.element.size = new Vector(100, 100)
    this.element.animateSize = this.defineAnimatedVectorSetter('size')

    super.setup(options)
  }

  bounds() {
    return new Vector(
      this.element.size.x * this.element.scale.x,
      this.element.size.y * this.element.scale.y
    )
  }

  extent() {
    return new Vector(
      this.element.size.x * this.element.scale.x,
      this.element.size.y * this.element.scale.y
    )
  }

  extentOffset() {
    return new Vector(0, 0)
  }

  async render(ctx: OffscreenCanvasRenderingContext2D) {
    const red = this.element.color.x * 255
    const blue = this.element.color.y * 255
    const green = this.element.color.z * 255
    const alpha = this.element.color.w

    const imageAspect = this.element.image.width / this.element.image.height

    const bounds = this.bounds()

    const ratioWidth = bounds.x
    const ratioHeight = bounds.y * imageAspect

    const bestWidth = Math.max(ratioWidth, ratioHeight)
    const bestHeight = bestWidth / imageAspect

    const offsetX = (bestWidth - bounds.x) / 2
    const offsetY = (bestHeight - bounds.y) / 2

    ctx.imageSmoothingEnabled = false

    ctx.fillStyle = `rgb(${red}, ${green}, ${blue})`
    ctx.fillRect(0, 0, bounds.x, bounds.y)
    ctx.globalCompositeOperation = 'multiply'

    ctx.drawImage(this.element.image, -offsetX, -offsetY, bestWidth, bestHeight)
    ctx.globalCompositeOperation = 'destination-atop'

    ctx.drawImage(this.element.image, -offsetX, -offsetY, bestWidth, bestHeight)
    ctx.globalCompositeOperation = 'destination-in'

    ctx.fillStyle = `rgb(1, 1, 1, ${alpha})`
    ctx.fillRect(0, 0, bounds.x, bounds.y)
  }
}

export class Text extends RenderingBuilder {
  setup(options: any) {
    this.element.text = ''
    this.element.font = 'JetBrainsMono'
    this.element.color = new Vector(0, 0, 0, 1)
    this.element.animateColor = this.defineAnimatedVectorSetter('color')
    this.element.outlineColor = new Vector(0, 0, 0, 0)
    this.element.animateOutlineColor =
      this.defineAnimatedVectorSetter('outlineColor')
    this.element.outlineWidth = 0
    this.element.animateOutlineWidth =
      this.defineAnimatedNumberSetter('outlineWidth')
    this.element.size = 24
    this.element.animateSize = this.defineAnimatedNumberSetter('size')

    super.setup(options)
  }

  bounds() {
    if (this.element.size == 0) return new Vector()

    const canvas = new OffscreenCanvas(0, 0)
    const ctx: OffscreenCanvasRenderingContext2D = <
      OffscreenCanvasRenderingContext2D
    >canvas.getContext('2d')
    ctx.font = `${this.element.size}px ${this.element.font}`
    const measure = ctx.measureText(this.element.text)

    return new Vector(
      measure.width * this.element.scale.x,
      (measure.actualBoundingBoxDescent + measure.actualBoundingBoxAscent) *
        this.element.scale.y
    )
  }

  extent() {
    if (this.element.size == 0) return new Vector()

    const canvas = new OffscreenCanvas(0, 0)
    const ctx: OffscreenCanvasRenderingContext2D = <
      OffscreenCanvasRenderingContext2D
    >canvas.getContext('2d')
    ctx.font = `${this.element.size}px ${this.element.font}`
    const measure = ctx.measureText(this.element.text)

    return new Vector(
      measure.width * this.element.scale.x,
      (measure.actualBoundingBoxDescent + measure.actualBoundingBoxAscent) *
        this.element.scale.y
    )
  }

  extentOffset() {
    return new Vector(0, 0)
  }

  async render(ctx: OffscreenCanvasRenderingContext2D) {
    const red = this.element.color.x * 255
    const blue = this.element.color.y * 255
    const green = this.element.color.z * 255
    const alpha = this.element.color.w

    const outlineRed = this.element.outlineColor.x * 255
    const outlineBlue = this.element.outlineColor.y * 255
    const outlineGreen = this.element.outlineColor.z * 255
    const outlineAlpha = this.element.outlineColor.w

    const bounds = this.bounds()

    if (Math.floor(bounds.x) == 0 || Math.floor(bounds.y) == 0) return

    const canvas = new OffscreenCanvas(
      bounds.x / this.element.scale.x,
      bounds.y / this.element.scale.y
    )
    const unscaledCtx: OffscreenCanvasRenderingContext2D = <
      OffscreenCanvasRenderingContext2D
    >canvas.getContext('2d')

    unscaledCtx.fillStyle = `rgba(${red},${blue},${green},${alpha})`
    unscaledCtx.strokeStyle = `rgba(${outlineRed},${outlineBlue},${outlineGreen},${outlineAlpha})`
    unscaledCtx.lineWidth = this.element.outlineWidth
    unscaledCtx.font = `${this.element.size}px ${this.element.font}`

    const measure = unscaledCtx.measureText(this.element.text)

    console.log(measure)

    unscaledCtx.fillText(
      this.element.text,
      0,
      measure.actualBoundingBoxAscent - 1
    )

    if (this.element.outlineWidth != 0)
      unscaledCtx.strokeText(
        this.element.text,
        0,
        measure.actualBoundingBoxAscent - 1
      )

    ctx.drawImage(canvas, 0, 0, bounds.x, bounds.y)
  }
}
