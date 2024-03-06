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

class Reactive<T> {
	constructor(public reactable: Reactable<T>) {}

	public get value(): T {
		return this.reactable()
	}

	public set value(value: OptionallyReactable<T>) {
		this.reactable = ensureReactable(value)
	}

	public *to(final: T, time: number, ease?: (t: number) => number): Generator {
		if (typeof final === 'number') {
			let initial = <number>this.value
			let finalFrame = Math.floor(time * 60)

			for (let f = 1; f <= finalFrame; f++) {
				let progress = ease === undefined ? f / finalFrame : ease(f / finalFrame)
				this.value = <T>(initial + (final - initial) * progress)

				yield* frame()
			}
		}
	}

	public *bounce(final: T, speed: number, ease?: (t: number) => number, times?: number): Generator {
		let loopCount = 0
		let forward = true

		let initial = this.value
		let finalFrame = Math.floor(speed * 60)

		while (times === undefined || loopCount < times) {
			if (typeof final === 'number') {
				for (let f = 1; f <= finalFrame; f++) {
					let progress = 0

					if (forward) {
						progress = ease === undefined ? f / finalFrame : ease(f / finalFrame)
					} else {
						progress = ease === undefined ? 1 - f / finalFrame : ease(1 - f / finalFrame)
					}

					this.value = <T>(<number>initial + (final - <number>initial) * progress)

					yield* frame()
				}
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
}

class Vector4 {
	constructor(public x: number, public y: number, public z: number, public w: number) {}
}

class Rect {
	public position: Reactive<Vector2> = react(new Vector2(0, 0))
	public size: Reactive<Vector2> = react(new Vector2(100, 100))
	public color: Reactive<Vector4> = react(new Vector4(1, 1, 1, 1))
	public radius: Reactive<number> = react(0)

	constructor(options: {
		position?: OptionallyReactable<Vector2>
		size?: OptionallyReactable<Vector2>
		color?: OptionallyReactable<Vector4>
		radius?: OptionallyReactable<number>
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
			size: this.size.value,
			color: this.color.value,
			radius: this.radius.value,
		}
	}
}

class Ellipse {
	public position: Reactive<Vector2> = react(new Vector2(0, 0))
	public size: Reactive<Vector2> = react(new Vector2(100, 100))
	public color: Reactive<Vector4> = react(new Vector4(1, 1, 1, 1))

	constructor(options: {
		position: OptionallyReactable<Vector2>
		size: OptionallyReactable<Vector2>
		color: OptionallyReactable<Vector4>
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
			size: this.size.value,
			color: this.color.value,
		}
	}
}

const elements: any[] = []

function add(element: any) {
	elements.push(element)

	return element
}

function clip(context: () => Generator<any, any, any>) {
	const generator = context()

	global.advance = function () {
		generator.next()

		Deno.core.ops.op_reset_frame()

		for (const element of elements) {
			Deno.core.ops.op_add_frame_element(element.to_static())
		}
	}
}

function ease(x: number): number {
	return x < 0.5 ? 4 * x * x * x : 1 - Math.pow(-2 * x + 2, 3) / 2
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

for (const [key, value] of Object.entries({
	Vector2,
	Vector4,
	Rect,
	Ellipse,

	react,

	add,
	clip,

	ease,
	linear,

	frame,
	seconds,
})) {
	global[key] = value
}
