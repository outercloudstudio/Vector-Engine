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

class Vector2 {
	constructor(public x: number, public y: number) {}
}

class Vector4 {
	constructor(public x: number, public y: number, public z: number, public w: number) {}
}

class Rect {
	constructor(
		public position: Vector2,
		public size: Vector2,
		public color: Vector4,
		public radius: number | (() => number)
	) {}

	public to_static() {
		return {
			type: 'Rect',
			position: this.position,
			size: this.size,
			color: this.color,
			radius: typeof this.radius === 'function' ? this.radius() : this.radius,
		}
	}
}

class Ellipse {
	constructor(
		public position: Vector2 | (() => Vector2),
		public size: Vector2,
		public color: Vector4
	) {}

	public to_static() {
		return {
			type: 'Ellipse',
			position: typeof this.position === 'function' ? this.position() : this.position,
			size: this.size,
			color: this.color,
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
