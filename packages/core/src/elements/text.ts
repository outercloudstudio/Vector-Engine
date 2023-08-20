import { ImageAsset } from '../assets/image'
import { MaybeReactor } from '../reactive'
import { Vector2 } from '../vector'
import { AnimatedReactiveProperty, Element, animatedReactiveProperty } from './element'

export class VectorText extends Element {
	public text: string = 'Text'
	public position: AnimatedReactiveProperty<Vector2> = animatedReactiveProperty<Vector2>(
		Vector2.zero()
	)
	public origin: AnimatedReactiveProperty<Vector2> = animatedReactiveProperty<Vector2>(
		Vector2.half()
	)
	public size: AnimatedReactiveProperty<number> = animatedReactiveProperty<number>(20)

	constructor(options?: {
		text?: string
		position?: MaybeReactor<Vector2>
		size?: MaybeReactor<number>
	}) {
		super()

		if (options === undefined) return

		if (options.text !== undefined) this.text = options.text
		if (options.position !== undefined) this.position(options.position)
		if (options.size !== undefined) this.size(options.size)
	}

	public async render(canvas: OffscreenCanvas) {
		const renderPosition = this.position().add(new Vector2(1920 / 2, 1080 / 2))
		const size = this.size()

		if (size === 0) return

		const context = canvas.getContext('2d')

		context.fillStyle = 'white'
		context.font = `${size}px "Segoe UI"`

		context.fillText(this.text, renderPosition.x, renderPosition.y)
	}
}
