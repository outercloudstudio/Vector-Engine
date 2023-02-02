import { Builder, RenderingBuilder } from '@/engine/Builders'
import { Scene } from '@/engine/Engine'

export class Element {
  builder: Builder

  isRendering: boolean = false

  scene: Scene

  constructor(scene: Scene, builder: typeof Builder, options: object) {
    this.scene = scene

    if (builder == undefined)
      throw new Error('You must specify a builder when creating an element')

    this.builder = new builder(this)
    this.builder.setup(options)
  }

  async render(parentCtx: CanvasRenderingContext2D) {
    if (!(this.builder instanceof RenderingBuilder))
      throw new Error('Can not render a non rendering builder')

    const canvas = document.createElement('canvas')
    const bounds = this.builder.bounds()
    canvas.width = bounds.x
    canvas.height = bounds.y
    const ctx = canvas.getContext('2d')!
    this.builder.render(ctx)

    const extent = this.builder.extent()
    const offset = this.builder.extentOffset()

    parentCtx.translate(this.position.x, this.position.y)
    parentCtx.rotate((this.rotation * Math.PI) / 180)
    parentCtx.translate(-offset.x, -offset.y)
    parentCtx.translate(-extent.x * this.origin.x, -extent.y * this.origin.y)

    parentCtx.drawImage(canvas, 0, 0)

    parentCtx.resetTransform()
  }

  [key: string]: any
}
