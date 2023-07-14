import { Element } from '../elements/element'
import { Asset } from './asset'

export function scene(generator: (scene: Scene) => Generator): Scene {
	const scene = new Scene(generator)

	return scene
}

export class Scene extends Asset {
	private context: Generator
	private elements: Element[] = []

	constructor(generator: (scene: Scene) => Generator) {
		super()

		this.context = generator(this)
		this.context.next()
	}

	nextFrame() {
		this.context.next()
	}

	render(canvas: OffscreenCanvas) {
		for (const element of this.elements) {
			element.render(canvas)
		}
	}

	add(element: Element) {
		this.elements.push(element)
	}
}
