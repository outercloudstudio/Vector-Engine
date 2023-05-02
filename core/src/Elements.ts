import { Scene } from './Scene'
import { lerp, uuid } from './Math'
import { Vector } from './Vector'
import { color } from './Color'
import { ensureReactive, OptionalReactor, reactive, Reactor, unreactive } from './Reactive'
import { Aside, waitFor } from './Aside'

export class Element {
	scene: Scene | undefined
	id: string = uuid()
}

function animatedVector(
	me: any,
	property: string,
	value?: OptionalReactor<Vector>,
	length?: number,
	mode?: any
): Generator | Vector | void {
	if (value === undefined) return me['_' + property]()

	if (length === undefined) {
		if ((<Reactor<Vector>>value).isReactive) {
			me['_' + property] = ensureReactive(value)
		} else {
			me['_' + property](value)
		}

		return
	}

	if (me.scene == undefined) throw new Error('Can not animate without being added to a scene!')

	const from = me['_' + property]()
	const to = unreactive(value)

	const aside = new Aside(function* () {
		for (let i = 1; i <= Math.ceil(length * me.scene.engine.frameRate - 1); i++) {
			me['_' + property](from.lerp(to, mode(i / Math.ceil(length * me.scene.engine.frameRate))))

			yield null
		}

		me['_' + property](to)
	})

	me.scene.addAside(aside)

	return waitFor(aside)
}

function animatedNumber(
	me: any,
	property: string,
	value?: OptionalReactor<number>,
	length?: number,
	mode?: any
): Generator | number | void {
	if (value === undefined) return me['_' + property]()

	if (length === undefined) {
		if ((<Reactor<number>>value).isReactive) {
			me['_' + property] = ensureReactive(value)
		} else {
			me['_' + property](value)
		}

		return
	}

	if (me.scene == undefined) throw new Error('Can not animate without being added to a scene!')

	const from = me['_' + property]()
	const to = unreactive(value)

	const aside = new Aside(function* () {
		for (let i = 1; i <= Math.ceil(length * me.scene.engine.frameRate) - 1; i++) {
			me['_' + property](lerp(from, to, mode(i / Math.ceil(length * me.scene.engine.frameRate))))

			yield null
		}

		me['_' + property](to)
	})

	me.scene.addAside(aside)

	return waitFor(aside)
}

function animatedString(me: any, property: string, value?: OptionalReactor<string>): string | void {
	if (value === undefined) return me['_' + property]()

	if ((<Reactor<string>>value).isReactive) {
		me['_' + property] = ensureReactive(value)
	} else {
		me['_' + property](value)
	}

	return
}

export class TransformElement extends Element {
	protected _position: Reactor<Vector> = reactive(new Vector(0, 0, 0, 0))
	protected _rotation: Reactor<number> = reactive(0)
	protected _scale: Reactor<Vector> = reactive(new Vector(1, 1))

	constructor(options?: {
		position?: OptionalReactor<Vector>
		rotation?: OptionalReactor<number>
		scale?: OptionalReactor<Vector>
	}) {
		super()

		if (options === undefined) return

		if (options.position !== undefined) this._position = ensureReactive(options.position)

		if (options.rotation !== undefined) this._rotation = ensureReactive(options.rotation)

		if (options.scale !== undefined) this._scale = ensureReactive(options.scale)
	}

	public position(): Vector
	public position(value: OptionalReactor<Vector>): void
	public position(value: OptionalReactor<Vector>, length: number, mode: any): Generator
	public position(
		value?: OptionalReactor<Vector>,
		length?: number,
		mode?: any
	): Generator | Vector | void {
		return animatedVector(this, 'position', value, length, mode)
	}

	public rotation(): number
	public rotation(value: OptionalReactor<number>): void
	public rotation(value: OptionalReactor<number>, length: number, mode: any): Generator
	public rotation(
		value?: OptionalReactor<number>,
		length?: number,
		mode?: any
	): Generator | number | void {
		return animatedNumber(this, 'rotation', value, length, mode)
	}

	public scale(): Vector
	public scale(value: OptionalReactor<Vector>): void
	public scale(value: OptionalReactor<Vector>, length: number, mode: any): Generator
	public scale(
		value?: OptionalReactor<Vector>,
		length?: number,
		mode?: any
	): Generator | Vector | void {
		return animatedVector(this, 'scale', value, length, mode)
	}
}

export class RenderElement extends TransformElement {
	protected _origin: Reactor<Vector> = reactive(new Vector(0.5, 0.5))
	public priority: number = 0

	constructor(options?: {
		position?: OptionalReactor<Vector>
		rotation?: OptionalReactor<number>
		scale?: OptionalReactor<Vector>
		origin?: OptionalReactor<Vector>
		priority?: number
	}) {
		super(options)

		if (options === undefined) return

		if (options.origin !== undefined) this._origin = ensureReactive(options.origin)

		if (options.priority !== undefined) this.priority = options.priority
	}

	async render(canvas: OffscreenCanvas) {
		throw new Error('Render method not implemented.')
	}

	public origin(): Vector
	public origin(value: OptionalReactor<Vector>): void
	public origin(value: OptionalReactor<Vector>, length: number, mode: any): Generator
	public origin(
		value?: OptionalReactor<Vector>,
		length?: number,
		mode?: any
	): Generator | Vector | void {
		return animatedVector(this, 'origin', value, length, mode)
	}
}

export class Rect extends RenderElement {
	protected _size: Reactor<Vector> = reactive(new Vector(100, 100))
	protected _color: Reactor<Vector> = reactive(color('#000000'))
	protected _outline: Reactor<Vector> = reactive(color('#000000'))
	protected _outlineWidth: Reactor<number> = reactive(0)
	protected _radius: Reactor<number> = reactive(0)

	constructor(options?: {
		position?: OptionalReactor<Vector>
		rotation?: OptionalReactor<number>
		scale?: OptionalReactor<Vector>
		origin?: OptionalReactor<Vector>
		priority?: number
		size?: OptionalReactor<Vector>
		color?: OptionalReactor<Vector>
		outline?: OptionalReactor<Vector>
		outlineWidth?: OptionalReactor<number>
		radius?: OptionalReactor<number>
	}) {
		super(options)

		if (options === undefined) return

		if (options.size !== undefined) this._size = ensureReactive(options.size)

		if (options.color !== undefined) this._color = ensureReactive(options.color)

		if (options.outline !== undefined) this._outline = ensureReactive(options.outline)

		if (options.outlineWidth !== undefined) this._outlineWidth = ensureReactive(options.outlineWidth)

		if (options.radius !== undefined) this._radius = ensureReactive(options.radius)
	}

	public size(): Vector
	public size(value: OptionalReactor<Vector>): void
	public size(value: OptionalReactor<Vector>, length: number, mode: any): Generator
	public size(
		value?: OptionalReactor<Vector>,
		length?: number,
		mode?: any
	): Generator | Vector | void {
		return animatedVector(this, 'size', value, length, mode)
	}

	public color(): Vector
	public color(value: OptionalReactor<Vector>): void
	public color(value: OptionalReactor<Vector>, length: number, mode: any): Generator
	public color(
		value?: OptionalReactor<Vector>,
		length?: number,
		mode?: any
	): Generator | Vector | void {
		return animatedVector(this, 'color', value, length, mode)
	}

	public outline(): Vector
	public outline(value: OptionalReactor<Vector>): void
	public outline(value: OptionalReactor<Vector>, length: number, mode: any): Generator
	public outline(
		value?: OptionalReactor<Vector>,
		length?: number,
		mode?: any
	): Generator | Vector | void {
		return animatedVector(this, 'outline', value, length, mode)
	}

	public outlineWidth(): number
	public outlineWidth(value: OptionalReactor<number>): void
	public outlineWidth(value: OptionalReactor<number>, length: number, mode: any): Generator
	public outlineWidth(
		value?: OptionalReactor<number>,
		length?: number,
		mode?: any
	): Generator | number | void {
		return animatedNumber(this, 'outlineWidth', value, length, mode)
	}

	public radius(): number
	public radius(value: OptionalReactor<number>): void
	public radius(value: OptionalReactor<number>, length: number, mode: any): Generator
	public radius(
		value?: OptionalReactor<number>,
		length?: number,
		mode?: any
	): Generator | number | void {
		return animatedNumber(this, 'radius', value, length, mode)
	}

	async render(canvas: OffscreenCanvas) {
		const ctx = canvas.getContext('2d')

		const position = this._position()
		const rotation = this._rotation()
		const scale = this._scale()
		const origin = this._origin()
		const size = this._size()
		const color = this._color()
		const outline = this._outline()
		const outlineWidth = this._outlineWidth()
		const radius = this._radius()

		ctx.translate(position.x, position.y)
		ctx.rotate((rotation * Math.PI) / 180)
		ctx.translate(-size.x * origin.x * scale.x, -size.y * origin.x * scale.y)

		const red = color.x * 255
		const blue = color.y * 255
		const green = color.z * 255
		const alpha = color.w

		const outlineRed = outline.x * 255
		const outlineBlue = outline.y * 255
		const outlineGreen = outline.z * 255
		const outlineAlpha = outline.w

		ctx.fillStyle = `rgba(${red},${blue},${green},${alpha})`
		ctx.strokeStyle = `rgba(${outlineRed},${outlineBlue},${outlineGreen},${outlineAlpha})`
		ctx.lineWidth = outlineWidth

		const width = size.x * scale.x
		const height = size.y * scale.y
		const radiusX = Math.min(Math.max(radius, 0) * scale.x, width / 2)
		const radiusY = Math.min(Math.max(radius, 0) * scale.y, height / 2)

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
		if (outlineWidth != 0) ctx.stroke()
	}
}

export class Ellipse extends RenderElement {
	protected _size: Reactor<Vector> = reactive(new Vector(100, 100))
	protected _color: Reactor<Vector> = reactive(color('#000000'))
	protected _outline: Reactor<Vector> = reactive(color('#000000'))
	protected _outlineWidth: Reactor<number> = reactive(0)

	constructor(options?: {
		position?: OptionalReactor<Vector>
		rotation?: OptionalReactor<number>
		scale?: OptionalReactor<Vector>
		origin?: OptionalReactor<Vector>
		priority?: number
		size?: OptionalReactor<Vector>
		color?: OptionalReactor<Vector>
		outline?: OptionalReactor<Vector>
		outlineWidth?: OptionalReactor<number>
	}) {
		super(options)

		if (options === undefined) return

		if (options.size !== undefined) this._size = ensureReactive(options.size)

		if (options.color !== undefined) this._color = ensureReactive(options.color)

		if (options.outline !== undefined) this._outline = ensureReactive(options.outline)

		if (options.outlineWidth !== undefined) this._outlineWidth = ensureReactive(options.outlineWidth)
	}

	public size(): Vector
	public size(value: OptionalReactor<Vector>): void
	public size(value: OptionalReactor<Vector>, length: number, mode: any): Generator
	public size(
		value?: OptionalReactor<Vector>,
		length?: number,
		mode?: any
	): Generator | Vector | void {
		return animatedVector(this, 'size', value, length, mode)
	}

	public color(): Vector
	public color(value: OptionalReactor<Vector>): void
	public color(value: OptionalReactor<Vector>, length: number, mode: any): Generator
	public color(
		value?: OptionalReactor<Vector>,
		length?: number,
		mode?: any
	): Generator | Vector | void {
		return animatedVector(this, 'color', value, length, mode)
	}

	public outline(): Vector
	public outline(value: OptionalReactor<Vector>): void
	public outline(value: OptionalReactor<Vector>, length: number, mode: any): Generator
	public outline(
		value?: OptionalReactor<Vector>,
		length?: number,
		mode?: any
	): Generator | Vector | void {
		return animatedVector(this, 'outline', value, length, mode)
	}

	public outlineWidth(): number
	public outlineWidth(value: OptionalReactor<number>): void
	public outlineWidth(value: OptionalReactor<number>, length: number, mode: any): Generator
	public outlineWidth(
		value?: OptionalReactor<number>,
		length?: number,
		mode?: any
	): Generator | number | void {
		return animatedNumber(this, 'outlineWidth', value, length, mode)
	}

	async render(canvas: OffscreenCanvas) {
		const ctx = canvas.getContext('2d')

		const position = this._position()
		const rotation = this._rotation()
		const scale = this._scale()
		const origin = this._origin()
		const size = this._size()
		const color = this._color()
		const outline = this._outline()
		const outlineWidth = this._outlineWidth()

		ctx.translate(position.x, position.y)
		ctx.rotate((rotation * Math.PI) / 180)
		ctx.translate(-size.x * origin.x, -size.y * origin.x)

		const red = color.x * 255
		const green = color.y * 255
		const blue = color.z * 255
		const alpha = color.w

		const outlineRed = outline.x * 255
		const outlineGreen = outline.y * 255
		const outlineBlue = outline.z * 255
		const outlineAlpha = outline.w

		ctx.fillStyle = `rgba(${red},${green},${blue},${alpha})`
		ctx.strokeStyle = `rgba(${outlineRed},${outlineGreen},${outlineBlue},${outlineAlpha})`
		ctx.lineWidth = outlineWidth

		ctx.beginPath()
		ctx.ellipse(
			(size.x / 2) * scale.x + outlineWidth / 2,
			(size.y / 2) * scale.y + outlineWidth / 2,
			(size.x / 2) * scale.x,
			(size.y / 2) * scale.y,
			0,
			0,
			2 * Math.PI
		)
		ctx.closePath()

		ctx.fill()
		if (outlineWidth != 0) ctx.stroke()
	}
}

export class VectorText extends RenderElement {
	public _text: Reactor<string> = reactive('Vector Engine')
	public font: string = 'Mulish'
	protected _size: Reactor<number> = reactive(100)
	protected _color: Reactor<Vector> = reactive(color('#000000'))
	protected _outline: Reactor<Vector> = reactive(color('#000000'))
	protected _outlineWidth: Reactor<number> = reactive(0)

	constructor(options?: {
		position?: OptionalReactor<Vector>
		rotation?: OptionalReactor<number>
		scale?: OptionalReactor<Vector>
		origin?: OptionalReactor<Vector>
		priority?: number
		text?: OptionalReactor<string>
		font?: string
		size?: OptionalReactor<number>
		color?: OptionalReactor<Vector>
		outline?: OptionalReactor<Vector>
		outlineWidth?: OptionalReactor<number>
	}) {
		super(options)

		if (options === undefined) return

		if (options.text !== undefined) this._text = ensureReactive(options.text)

		if (options.font !== undefined) this.font = options.font

		if (options.size !== undefined) this._size = ensureReactive(options.size)

		if (options.color !== undefined) this._color = ensureReactive(options.color)

		if (options.outline !== undefined) this._outline = ensureReactive(options.outline)

		if (options.outlineWidth !== undefined) this._outlineWidth = ensureReactive(options.outlineWidth)
	}

	public text(): string
	public text(value: OptionalReactor<string>): void
	public text(value?: OptionalReactor<string>): string | void {
		return animatedString(this, 'text', value)
	}

	public size(): number
	public size(value: OptionalReactor<number>): void
	public size(value: OptionalReactor<number>, length: number, mode: any): Generator
	public size(
		value?: OptionalReactor<number>,
		length?: number,
		mode?: any
	): Generator | number | void {
		return animatedNumber(this, 'size', value, length, mode)
	}

	public color(): Vector
	public color(value: OptionalReactor<Vector>): void
	public color(value: OptionalReactor<Vector>, length: number, mode: any): Generator
	public color(
		value?: OptionalReactor<Vector>,
		length?: number,
		mode?: any
	): Generator | Vector | void {
		return animatedVector(this, 'color', value, length, mode)
	}

	public outline(): Vector
	public outline(value: OptionalReactor<Vector>): void
	public outline(value: OptionalReactor<Vector>, length: number, mode: any): Generator
	public outline(
		value?: OptionalReactor<Vector>,
		length?: number,
		mode?: any
	): Generator | Vector | void {
		return animatedVector(this, 'outline', value, length, mode)
	}

	public outlineWidth(): number
	public outlineWidth(value: OptionalReactor<number>): void
	public outlineWidth(value: OptionalReactor<number>, length: number, mode: any): Generator
	public outlineWidth(
		value?: OptionalReactor<number>,
		length?: number,
		mode?: any
	): Generator | number | void {
		return animatedNumber(this, 'outlineWidth', value, length, mode)
	}

	async render(canvas: OffscreenCanvas) {
		const ctx = canvas.getContext('2d')

		const position = this._position()
		const rotation = this._rotation()
		const scale = this._scale()
		const origin = this._origin()
		const text = this._text()
		const size = this._size() * scale.x
		const color = this._color()
		const outline = this._outline()
		const outlineWidth = this._outlineWidth()

		ctx.translate(position.x, position.y)
		ctx.rotate((rotation * Math.PI) / 180)

		ctx.font = `${size}px ${this.font}`

		const measure = ctx.measureText(text)

		ctx.translate(-measure.width * origin.x, -measure.actualBoundingBoxAscent * origin.y)
		ctx.scale(1, -1)

		const red = color.x * 255
		const green = color.y * 255
		const blue = color.z * 255
		const alpha = color.w

		const outlineRed = outline.x * 255
		const outlineGreen = outline.y * 255
		const outlineBlue = outline.z * 255
		const outlineAlpha = outline.w

		ctx.fillStyle = `rgba(${red},${green},${blue},${alpha})`
		ctx.strokeStyle = `rgba(${outlineRed},${outlineGreen},${outlineBlue},${outlineAlpha})`
		ctx.lineWidth = outlineWidth

		ctx.fillText(text, 0, 0)
		if (outlineWidth != 0) ctx.strokeText(text, 0, 0)
	}
}

export class VectorImage extends RenderElement {
	public image: HTMLImageElement = undefined
	protected _size: Reactor<Vector> = reactive(new Vector(100, 100))
	protected _color: Reactor<Vector> = reactive(color('#ffffffff'))

	constructor(options?: {
		position?: OptionalReactor<Vector>
		rotation?: OptionalReactor<number>
		scale?: OptionalReactor<Vector>
		origin?: OptionalReactor<Vector>
		priority?: number
		image?: HTMLImageElement
		size?: OptionalReactor<Vector>
		color?: OptionalReactor<Vector>
	}) {
		super(options)

		if (options === undefined) return

		if (options.image !== undefined) this.image = options.image

		if (options.size !== undefined) this._size = ensureReactive(options.size)

		if (options.color !== undefined) this._color = ensureReactive(options.color)
	}

	public size(): Vector
	public size(value: OptionalReactor<Vector>): void
	public size(value: OptionalReactor<Vector>, length: number, mode: any): Generator
	public size(
		value?: OptionalReactor<Vector>,
		length?: number,
		mode?: any
	): Generator | Vector | void {
		return animatedVector(this, 'size', value, length, mode)
	}

	public color(): Vector
	public color(value: OptionalReactor<Vector>): void
	public color(value: OptionalReactor<Vector>, length: number, mode: any): Generator
	public color(
		value?: OptionalReactor<Vector>,
		length?: number,
		mode?: any
	): Generator | Vector | void {
		return animatedVector(this, 'color', value, length, mode)
	}

	async render(canvas: OffscreenCanvas) {
		const ctx = canvas.getContext('2d')

		const position = this._position()
		const rotation = this._rotation()
		const scale = this._scale()
		const origin = this._origin()
		const size = this._size()
		const color = this._color()

		ctx.translate(position.x, position.y)
		ctx.rotate((rotation * Math.PI) / 180)

		const width = size.x * scale.x
		const height = size.y * scale.y

		const imageAspect = this.image.width / this.image.height

		const red = color.x * 255
		const blue = color.y * 255
		const green = color.y * 255
		const alpha = color.w

		const bestWidth = Math.max(width, height * imageAspect)
		const bestHeight = bestWidth / imageAspect

		const unscaledWidth = Math.ceil((this.image.width * width) / bestWidth)
		const unscaledHeight = Math.ceil((this.image.height * height) / bestHeight)

		ctx.scale(1, -1)

		ctx.imageSmoothingEnabled = false

		const colorCanvas = new OffscreenCanvas(unscaledWidth, unscaledHeight)
		const colorCtx = colorCanvas.getContext('2d')

		colorCtx.imageSmoothingEnabled = false

		colorCtx.fillStyle = `rgb(${red}, ${green}, ${blue})`
		colorCtx.fillRect(0, 0, unscaledWidth, unscaledHeight)
		colorCtx.globalCompositeOperation = 'multiply'

		colorCtx.drawImage(
			this.image,
			-(this.image.width - unscaledWidth) / 2,
			-(this.image.height - unscaledHeight) / 2,
			this.image.width,
			this.image.height
		)

		colorCtx.globalCompositeOperation = 'destination-atop'

		colorCtx.drawImage(
			this.image,
			-(this.image.width - unscaledWidth) / 2,
			-(this.image.height - unscaledHeight) / 2,
			this.image.width,
			this.image.height
		)
		colorCtx.globalCompositeOperation = 'destination-in'

		colorCtx.fillStyle = `rgb(1, 1, 1, ${alpha})`
		colorCtx.fillRect(0, 0, unscaledWidth, unscaledHeight)

		ctx.drawImage(colorCanvas, -width * origin.x, -height * (1 - origin.y), width, height)

		ctx.imageSmoothingEnabled = true
	}
}

export class VectorVideo extends RenderElement {
	video: HTMLVideoElement = undefined
	protected _size: Reactor<Vector> = reactive(new Vector(100, 100))
	protected _color: Reactor<Vector> = reactive(color('#FFFFFFFF'))
	protected _time: Reactor<number> = reactive(0)

	constructor(options?: {
		position?: OptionalReactor<Vector>
		rotation?: OptionalReactor<number>
		scale?: OptionalReactor<Vector>
		origin?: OptionalReactor<Vector>
		priority?: number
		video?: HTMLVideoElement
		size?: OptionalReactor<Vector>
		color?: OptionalReactor<Vector>
		time?: OptionalReactor<number>
	}) {
		super(options)

		if (options === undefined) return

		if (options.video !== undefined) this.video = options.video

		if (options.size !== undefined) this._size = ensureReactive(options.size)

		if (options.color !== undefined) this._color = ensureReactive(options.color)

		if (options.time !== undefined) this._time = ensureReactive(options.time)
	}

	public async *play() {
		while (this.time() < this.video.duration) {
			this.time(this.time() + 1 / this.scene.engine.frameRate)

			yield null
		}
	}

	public size(): Vector
	public size(value: OptionalReactor<Vector>): void
	public size(value: OptionalReactor<Vector>, length: number, mode: any): Generator
	public size(
		value?: OptionalReactor<Vector>,
		length?: number,
		mode?: any
	): Generator | Vector | void {
		return animatedVector(this, 'size', value, length, mode)
	}

	public color(): Vector
	public color(value: OptionalReactor<Vector>): void
	public color(value: OptionalReactor<Vector>, length: number, mode: any): Generator
	public color(
		value?: OptionalReactor<Vector>,
		length?: number,
		mode?: any
	): Generator | Vector | void {
		return animatedVector(this, 'color', value, length, mode)
	}

	public time(): number
	public time(value: OptionalReactor<number>): void
	public time(value: OptionalReactor<number>, length: number, mode: any): Generator
	public time(
		value?: OptionalReactor<number>,
		length?: number,
		mode?: any
	): Generator | number | void {
		return animatedNumber(this, 'time', value, length, mode)
	}

	async render(canvas: OffscreenCanvas) {
		const color = this._color()

		if (color.w === 0) return

		if (this.time() >= this.video.duration) return

		await new Promise<void>(res => {
			if (this.time() === this.video.currentTime && this.video.currentTime !== 0) return

			this.video.currentTime = this.time()

			this.video.addEventListener('seeked', () => res())
		})

		const ctx = canvas.getContext('2d')

		const position = this._position()
		const rotation = this._rotation()
		const scale = this._scale()
		const origin = this._origin()
		const size = this._size()

		ctx.translate(position.x, position.y)
		ctx.rotate((rotation * Math.PI) / 180)

		const width = size.x * scale.x
		const height = size.y * scale.y

		const imageAspect = this.video.videoWidth / this.video.videoHeight

		const bestWidth = Math.max(width, height * imageAspect)
		const bestHeight = bestWidth / imageAspect

		ctx.scale(1, -1)

		ctx.imageSmoothingEnabled = false

		const red = color.x * 255
		const blue = color.y * 255
		const green = color.y * 255
		const alpha = color.w

		const colorCanvas = new OffscreenCanvas(width, height)
		const colorCtx = colorCanvas.getContext('2d')
		colorCtx.imageSmoothingEnabled = false

		colorCtx.fillStyle = `rgb(${red}, ${green}, ${blue})`
		colorCtx.fillRect(0, 0, width, height)
		colorCtx.globalCompositeOperation = 'multiply'

		colorCtx.drawImage(
			this.video,
			(width - bestWidth) / 2,
			(height - bestHeight) / 2,
			bestWidth,
			bestHeight
		)
		colorCtx.globalCompositeOperation = 'destination-atop'

		colorCtx.drawImage(
			this.video,
			(width - bestWidth) / 2,
			(height - bestHeight) / 2,
			bestWidth,
			bestHeight
		)
		colorCtx.globalCompositeOperation = 'destination-in'

		colorCtx.fillStyle = `rgb(1, 1, 1, ${alpha})`
		colorCtx.fillRect(0, 0, width, height)

		ctx.drawImage(colorCanvas, -width * origin.x, -height * origin.y)

		ctx.imageSmoothingEnabled = true
	}
}
