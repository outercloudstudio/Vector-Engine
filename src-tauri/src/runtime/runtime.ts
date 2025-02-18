const global: any = globalThis

function argsToMessage(...args: any[]) {
	return args.map(arg => JSON.stringify(arg)).join(' ')
}

global.console = {
	log: (...args: any[]) => {
		Deno.core.print(`[out]: ${argsToMessage(...args)}\n`, false)
	},
	error: (...args: any[]) => {
		Deno.core.print(`[err]: ${argsToMessage(...args)}\n`, true)
	},
}

type Reactable<T> = () => T
type OptionallyReactable<T> = T | Reactable<T>

function ensureReactable<T>(value: OptionallyReactable<T>): Reactable<T> {
	if (typeof value !== 'function') return () => <T>value

	return <Reactable<T>>value
}

function lerp<T>(a: T, b: T, t: number): T {
	if (typeof a === 'number' && typeof b === 'number') {
		return <T>(a + (b - a) * t)
	} else if (a instanceof Vector2 && b instanceof Vector2) {
		return <T>new Vector2(lerp(a.x, b.x, t), lerp(a.y, b.y, t))
	} else if (a instanceof Vector4 && b instanceof Vector4) {
		return <T>new Vector4(lerp(a.x, b.x, t), lerp(a.y, b.y, t), lerp(a.z, b.z, t), lerp(a.w, b.w, t))
	}

	return a
}

class Reactive<T> {
	constructor(public reactable: Reactable<T>) {}

	public get value(): T {
		return this.reactable()
	}

	public set value(value: OptionallyReactable<T>) {
		this.reactable = ensureReactable(value)
	}

	public *to(final: T, time: number, ease?: (t: number) => number): Generator {
		let initial = this.value
		let finalFrame = Math.floor(time * 60)

		for (let f = 1; f <= finalFrame; f++) {
			let progress = ease === undefined ? f / finalFrame : ease(f / finalFrame)
			this.value = <T>lerp(initial, final, progress)

			yield* frame()
		}

		this.value = final
	}

	public *bounce(final: T, speed: number, ease?: (t: number) => number, times?: number): Generator {
		let loopCount = 0
		let forward = true

		let initial = this.value
		let finalFrame = Math.floor(speed * 60)

		while (times === undefined || loopCount < times) {
			for (let f = 1; f <= finalFrame; f++) {
				let progress = 0

				if (forward) {
					progress = ease === undefined ? f / finalFrame : ease(f / finalFrame)
				} else {
					progress = ease === undefined ? 1 - f / finalFrame : ease(1 - f / finalFrame)
				}

				this.value = <T>lerp(initial, final, progress)

				yield* frame()
			}

			forward = !forward

			loopCount++
		}
	}
}

function react<T>(value: OptionallyReactable<T>): Reactive<T> {
	return new Reactive<T>(ensureReactable(value))
}

class Vector2 {
	constructor(public x: number, public y: number) {}

	public add(b: Vector2) {
		return new Vector2(this.x + b.x, this.y + b.y)
	}
}

class Vector4 {
	constructor(public x: number, public y: number, public z: number, public w: number) {}
}

class Rect {
	public position: Reactive<Vector2> = react(new Vector2(0, 0))
	public origin: Reactive<Vector2> = react(new Vector2(0.5, 0.5))
	public size: Reactive<Vector2> = react(new Vector2(100, 100))
	public rotation: Reactive<number> = react(0)
	public color: Reactive<Vector4> = react(new Vector4(1, 1, 1, 1))
	public radius: Reactive<number> = react(0)
	public order: Reactive<number> = react(0)

	constructor(options: {
		position?: OptionallyReactable<Vector2>
		origin?: OptionallyReactable<Vector2>
		size?: OptionallyReactable<Vector2>
		rotation?: OptionallyReactable<number>
		color?: OptionallyReactable<Vector4>
		radius?: OptionallyReactable<number>
		order?: OptionallyReactable<number>
	}) {
		for (const key of Object.keys(options)) {
			//@ts-ignore
			this[key] = react(options[key])
		}
	}

	public to_static() {
		return {
			type: 'Rect',
			position: this.position.value,
			origin: this.origin.value,
			size: this.size.value,
			rotation: this.rotation.value,
			color: this.color.value,
			radius: this.radius.value,
			order: this.order.value,
		}
	}
}

class Ellipse {
	public position: Reactive<Vector2> = react(new Vector2(0, 0))
	public origin: Reactive<Vector2> = react(new Vector2(0.5, 0.5))
	public size: Reactive<Vector2> = react(new Vector2(100, 100))
	public color: Reactive<Vector4> = react(new Vector4(1, 1, 1, 1))
	public order: Reactive<number> = react(0)

	constructor(options: {
		position?: OptionallyReactable<Vector2>
		origin?: OptionallyReactable<Vector2>
		size?: OptionallyReactable<Vector2>
		color?: OptionallyReactable<Vector4>
		order?: OptionallyReactable<number>
	}) {
		for (const key of Object.keys(options)) {
			//@ts-ignore
			this[key] = react(options[key])
		}
	}

	public to_static() {
		return {
			type: 'Ellipse',
			position: this.position.value,
			origin: this.origin.value,
			size: this.size.value,
			color: this.color.value,
			order: this.order.value,
		}
	}
}

class Clip {
	public clip: Reactive<string> = react('')
	public frame: Reactive<number> = react(0)
	public position: Reactive<Vector2> = react(new Vector2(0, 0))
	public origin: Reactive<Vector2> = react(new Vector2(0.5, 0.5))
	public size: Reactive<Vector2> = react(new Vector2(100, 100))
	public rotation: Reactive<number> = react(0)
	public color: Reactive<Vector4> = react(new Vector4(1, 1, 1, 1))
	public order: Reactive<number> = react(0)

	constructor(options: {
		clip?: OptionallyReactable<string>
		frame?: OptionallyReactable<number>
		position?: OptionallyReactable<Vector2>
		origin?: OptionallyReactable<Vector2>
		size?: OptionallyReactable<Vector2>
		rotation?: OptionallyReactable<number>
		color?: OptionallyReactable<Vector4>
		order?: OptionallyReactable<number>
	}) {
		for (const key of Object.keys(options)) {
			//@ts-ignore
			this[key] = react(options[key])
		}
	}

	public to_static() {
		return {
			type: 'Clip',
			clip: this.clip.value,
			frame: this.frame.value,
			position: this.position.value,
			origin: this.origin.value,
			size: this.size.value,
			rotation: this.rotation.value,
			color: this.color.value,
			order: this.order.value,
		}
	}
}

class FontAtlas {
	constructor(
		public path: string,
		public rows: number,
		public columns: number,
		public dropdown: number,
		public spacing: number,
		public characters: string,
		public widthOverrides: Record<string, number>
	) {}
}

class VectText {
	public text: Reactive<string> = react('')
	public font: FontAtlas = undefined!
	public position: Reactive<Vector2> = react(new Vector2(0, 0))
	public origin: Reactive<Vector2> = react(new Vector2(0.5, 0.5))
	public characterSize: Reactive<number> = react(100)
	public rotation: Reactive<number> = react(0)
	public color: Reactive<Vector4> = react(new Vector4(1, 1, 1, 1))
	public order: Reactive<number> = react(0)

	constructor(options: {
		text?: OptionallyReactable<string>
		font?: FontAtlas
		position?: OptionallyReactable<Vector2>
		origin?: OptionallyReactable<Vector2>
		characterSize?: OptionallyReactable<number>
		rotation?: OptionallyReactable<number>
		color?: OptionallyReactable<Vector4>
		order?: OptionallyReactable<number>
	}) {
		for (const key of Object.keys(options)) {
			if (key === 'font' && options.font) {
				this.font = options.font

				continue
			}

			//@ts-ignore
			this[key] = react(options[key])
		}
	}

	public size: Reactive<Vector2> = react(() => {
		let width = 0

		for (const character of this.text.value) {
			if (this.font.widthOverrides[character] !== undefined) {
				width += this.characterSize.value * this.font.widthOverrides[character]
			} else {
				width += this.characterSize.value * this.font.spacing
			}
		}

		return new Vector2(width, this.characterSize.value)
	})

	public to_static() {
		return {
			type: 'Text',
			text: this.text.value,
			font: this.font,
			position: this.position.value,
			origin: this.origin.value,
			size: this.characterSize.value,
			rotation: this.rotation.value,
			color: this.color.value,
			order: this.order.value,
		}
	}
}

const elements: any[] = []

function add<T>(element: T): T {
	elements.push(element)

	return element
}

function remove<T>(element: T): T {
	elements.splice(elements.indexOf(element), 1)

	return element
}

function _updateFrame() {
	for (const element of elements) {
		Deno.core.ops.op_add_frame_element(element.to_static())
	}
}

function clip(context: () => Generator<any, any, any>) {
	const generator = context()

	Deno.core.ops.op_add_context(generator)
}

function ease(x: number): number {
	return x < 0.5 ? 4 * x * x * x : 1 - Math.pow(-2 * x + 2, 3) / 2
}

function easeIn(x: number): number {
	return x * x * x
}

function easeOut(x: number): number {
	return 1 - Math.pow(1 - x, 3)
}

function easeOutQuint(x: number): number {
	return 1 - Math.pow(1 - x, 5)
}

function easeOutBack(x: number): number {
	const c1 = 1.70158
	const c3 = c1 + 1

	return 1 + c3 * Math.pow(x - 1, 3) + c1 * Math.pow(x - 1, 2)
}

function linear(x: number): number {
	return x
}

function* frame() {
	yield null
}

function* seconds(time: number) {
	let finalFrame = Math.floor(time * 60)

	for (let f = 1; f <= finalFrame; f++) {
		yield* frame()
	}
}

function rgba(r: number, g: number, b: number, a: number): Vector4 {
	return new Vector4(r / 255, g / 255, b / 255, a)
}

for (const [key, value] of Object.entries({
	Vector2,
	Vector4,

	FontAtlas,

	Rect,
	Ellipse,
	Clip,
	VectText,

	react,

	add,
	remove,
	clip,

	ease,
	easeIn,
	easeOut,
	easeOutBack,
	linear,
	easeOutQuint,

	frame,
	seconds,

	rgba,

	_updateFrame,
})) {
	global[key] = value
}
