import { Element } from '../elements/element'
import { Asset } from './asset'

export function scene(generator: (scene: Scene) => Generator): () => Scene {
	return () => new Scene(generator)
}

export class Scene extends Asset {
	private generator: (scene: Scene) => Generator
	private context: Generator
	private elements: Element[] = []
	private internalFrame: number = 0

	constructor(generator: (scene: Scene) => Generator) {
		super()

		this.generator = generator
		this.context = this.generator(this)
		this.context.next()
	}

	toFrame(frame: number) {
		if (frame < this.internalFrame) {
			console.warn('Context reload!')

			this.elements = []
			this.context = this.generator(this)
			this.context.next()
			this.internalFrame = 0
		}

		for (; this.internalFrame < frame; this.internalFrame++) {
			this.context.next()
		}
	}

	async render(canvas: OffscreenCanvas) {
		for (const element of this.elements) {
			await element.render(canvas)
		}
	}

	add(element: Element) {
		this.elements.push(element)
	}
}
