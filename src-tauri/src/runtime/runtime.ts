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
	private readonly type = 'Rect'

	constructor(public position: Vector2, public size: Vector2, public color: Vector4) {}
}

for (const [key, value] of Object.entries({
	Vector2,
	Vector4,
	Rect,
})) {
	global[key] = value
}

const elements: any[] = []

global.add = function (element: any) {
	elements.push(element)

	return element
}

global.clip = function (context: GeneratorFunction) {
	const generator = context()

	global.advance = function () {
		generator.next()

		Deno.core.ops.op_reset_frame()

		for (const element of elements) {
			Deno.core.ops.op_add_frame_element(element)
		}
	}
}
