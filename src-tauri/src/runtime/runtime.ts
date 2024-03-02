;(globalThis => {
	function argsToMessage(...args) {
		return args.map(arg => JSON.stringify(arg)).join(' ')
	}

	;(<any>globalThis).console = {
		log: (...args) => {
			Deno.core.print(`[out]: ${argsToMessage(...args)}\n`, false)
		},
		error: (...args) => {
			Deno.core.print(`[err]: ${argsToMessage(...args)}\n`, true)
		},
	}

	class Vector2 {
		constructor(public x: number, public y: number) {}
	}

	class Rect {
		private readonly type = 'Rect'

		constructor(public position: Vector2, public size: Vector2) {}
	}

	;(<any>globalThis).Vector2 = Vector2
	;(<any>globalThis).Rect = Rect
})(globalThis)
;(globalThis => {
	const elements: any[] = []

	globalThis.add = function (element: any) {
		elements.push(element)

		return element
	}

	globalThis.clip = function (context: GeneratorFunction) {
		const generator = context()

		globalThis.advance = function () {
			generator.next()

			Deno.core.ops.op_reset_frame()

			for (const element of elements) {
				Deno.core.ops.op_add_frame_element(element)
			}
		}
	}
})(globalThis)
