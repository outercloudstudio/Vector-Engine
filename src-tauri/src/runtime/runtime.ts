;(globalThis => {
	//@ts-ignore
	const core = Deno.core

	function argsToMessage(...args) {
		return args.map(arg => JSON.stringify(arg)).join(' ')
	}

	;(<any>globalThis).console = {
		log: (...args) => {
			core.print(`[out]: ${argsToMessage(...args)}\n`, false)
		},
		error: (...args) => {
			core.print(`[err]: ${argsToMessage(...args)}\n`, true)
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
	;(<any>globalThis).add = function (element: any) {
		core.ops.op_add_element(element)

		return element
	}
})(globalThis)
