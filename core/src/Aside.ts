import { isGenerator } from './Utils'

export class Aside {
	context: AsyncGenerator | Generator
	done: boolean = false

	constructor(context: (() => Generator) | (() => AsyncGenerator) | AsyncGenerator | Generator) {
		if (isGenerator(context)) {
			this.context = <AsyncGenerator | Generator>context
		} else {
			this.context = (<(() => Generator) | (() => AsyncGenerator)>context)()
		}
	}

	async next() {
		const result = await this.context.next()

		if (result.done) this.done = true
	}
}

export function* waitFor(aside: Aside) {
	while (!aside.done) {
		yield null
	}
}

export async function* loop(contextLambda: (() => Generator) | (() => AsyncGenerator)) {
	while (true) {
		const context = contextLambda()

		yield* await context
	}
}

export async function* waitWhile(condition: (() => boolean) | (() => Promise<boolean>)) {
	while (await condition()) yield null
}
