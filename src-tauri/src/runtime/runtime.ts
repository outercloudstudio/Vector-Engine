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

for (const [key, value] of Object.entries({
	Vector2,
	Vector4,
	Rect,
	Ellipse,

	add,
	clip,
})) {
	global[key] = value
}
