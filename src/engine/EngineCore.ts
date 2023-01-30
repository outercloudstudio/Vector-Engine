export class Vector {
  x: number
  y: number
  z: number
  w: number

  constructor(x?: number, y?: number, z?: number, w?: number) {
    this.x = x || 0
    this.y = y || 0
    this.z = z || 0
    this.w = w || 0
  }

  add(a: Vector): Vector {
    return new Vector(this.x + a.x, this.y + a.y, this.z + a.z, this.w + a.w)
  }

  subtract(a: Vector): Vector {
    return new Vector(this.x - a.x, this.y - a.y, this.z - a.z, this.w - a.w)
  }

  multiply(a: Vector): Vector {
    return new Vector(this.x * a.x, this.y * a.y, this.z * a.z, this.w * a.w)
  }

  divide(a: Vector): Vector {
    return new Vector(this.x / a.x, this.y / a.y, this.z / a.z, this.w / a.w)
  }

  lerp(a: Vector, t: number): Vector {
    return new Vector(
      this.x + (a.x - this.x) * t,
      this.y + (a.y - this.y) * t,
      this.z + (a.z - this.z) * t,
      this.w + (a.w - this.w) * t
    )
  }

  clone(): Vector {
    return new Vector(this.x, this.y, this.z, this.w)
  }

  dot(a: Vector): number {
    return this.x * a.x + this.y * a.y + this.z * a.z + this.w * a.w
  }

  magnitude(): number {
    return Math.sqrt(
      this.x * this.x + this.y * this.y + this.z * this.z + this.w * this.w
    )
  }
}

export class Element {
  builder: any

  isRendering: boolean = false

  constructor(builder: any, options: any) {
    if (builder == undefined)
      throw new Error('You must specify a builder when creating an element')

    this.builder = new builder(this)
    this.builder.setup(options)
  }

  async render(parentCtx: any) {
    if (this.builder.render == undefined)
      throw new Error('Can not render a non rendering builder (missing render)')

    if (this.builder.bounds == undefined)
      throw new Error('Can not render a non rendering builder (missing bounds)')

    if (this.builder.offset == undefined)
      throw new Error('Can not render a non rendering builder (missing offset)')

    if (this.size == undefined)
      throw new Error('Can not render a non rendering builder (missing size)')

    if (this.size == undefined)
      throw new Error('Can not render a non rendering builder (missing origin)')

    if (this.position == undefined)
      throw new Error(
        'Can not render a non rendering builder (missing position)'
      )

    if (this.rotation == undefined)
      throw new Error(
        'Can not render a non rendering builder (missing rotation)'
      )

    if (this.priority == undefined)
      throw new Error(
        'Can not render a non rendering builder (missing priority)'
      )

    const canvas = document.createElement('canvas')
    const bounds = this.builder.bounds()
    canvas.width = bounds.x
    canvas.height = bounds.y
    const ctx = canvas.getContext('2d')!
    this.builder.render(ctx)

    parentCtx.translate(this.position.x, this.position.y)
    const offset = this.builder.offset()
    parentCtx.translate(-offset.x, -offset.y)
    parentCtx.rotate((this.rotation * Math.PI) / 180)
    parentCtx.translate(
      -this.size.x * this.origin.x,
      -this.size.y * this.origin.y
    )

    parentCtx.drawImage(canvas, 0, 0)

    parentCtx.resetTransform()
  }

  [key: string]: any
}
