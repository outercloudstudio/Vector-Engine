import { Element } from '../elements/element'
import { Asset } from './asset'

export function scene(generator: (scene: Scene) => Generator): () => Scene {
	return () => new Scene(generator)
}

type FinishableGenerator = Generator & { done?: boolean }

export class Scene extends Asset {
	private generator: (scene: Scene) => Generator
	private context: Generator

	private additionalContexts: FinishableGenerator[] = []

	private elements: Element[] = []

	private internalFrame: number = 0

	constructor(generator: (scene: Scene) => Generator) {
		super()

		this.generator = generator
		this.context = this.generator(this)

		this.next()
	}

	private next() {
		this.context.next()

		for (const context of this.additionalContexts) {
			const result = context.next()

			if (result.done) context.done = true
		}

		this.additionalContexts = this.additionalContexts.filter(context => !context.done)
	}

	public toFrame(frame: number) {
		if (frame < this.internalFrame) {
			console.warn('Context reload!')

			this.elements = []
			this.additionalContexts = []
			this.context = this.generator(this)
			this.context.next()
			this.internalFrame = 0
		}

		for (; this.internalFrame < frame; this.internalFrame++) {
			this.next()
		}
	}

	public async render(canvas: OffscreenCanvas) {
		for (const element of this.elements) {
			await element.render(canvas)
		}
	}

	public add<ElementType extends Element>(element: ElementType): ElementType {
		this.elements.push(element)

		element.scene = this

		return element
	}

	public aside(generator: (() => Generator) | Generator) {
		this.additionalContexts.push(typeof generator === 'function' ? generator() : generator)
	}
}
