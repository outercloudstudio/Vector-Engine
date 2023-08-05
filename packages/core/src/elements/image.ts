import { ImageAsset } from '../assets/image'
import { MaybeReactor } from '../reactive'
import { Vector2 } from '../vector'
import { AnimatedReactiveProperty, Element, animatedReactiveProperty } from './element'

export class VectorImage extends Element {
	public image: ImageAsset
	public position: AnimatedReactiveProperty<Vector2> = animatedReactiveProperty<Vector2>(
		Vector2.zero()
	)
	public size: AnimatedReactiveProperty<Vector2> = animatedReactiveProperty<Vector2>(Vector2.zero())

	constructor(options?: {
		image?: () => ImageAsset
		position?: MaybeReactor<Vector2>
		size?: MaybeReactor<Vector2>
	}) {
		super()

		if (options === undefined) return

		if (options.image !== undefined) this.image = options.image()
		if (options.position !== undefined) this.position(options.position)
		if (options.size !== undefined) this.size(options.size)
	}

	public async render(canvas: OffscreenCanvas) {
		this.image.position = this.position()
		this.image.size = this.size()

		await this.image.render(canvas)

		for (const filter of this.filters) {
			await filter.render(canvas)
		}
	}
}
