import { ImageAsset } from '../assets/image'
import { MaybeReactor } from '../reactive'
import { Bounds, Vector2 } from '../vector'
import { AnimatedReactiveProperty, Element, animatedReactiveProperty } from './element'

export class VectorImage extends Element {
	public image: ImageAsset
	public position: AnimatedReactiveProperty<Vector2> = animatedReactiveProperty<Vector2>(
		Vector2.zero()
	)
	public origin: AnimatedReactiveProperty<Vector2> = animatedReactiveProperty<Vector2>(
		Vector2.half()
	)
	public size: AnimatedReactiveProperty<Vector2> = animatedReactiveProperty<Vector2>(Vector2.zero())

	constructor(options?: {
		image?: () => ImageAsset
		position?: MaybeReactor<Vector2>
		origin?: MaybeReactor<Vector2>
		size?: MaybeReactor<Vector2>
	}) {
		super()

		if (options === undefined) return

		if (options.image !== undefined) this.image = options.image()
		if (options.position !== undefined) this.position(options.position)
		if (options.origin !== undefined) this.origin(options.origin)
		if (options.size !== undefined) this.size(options.size)
	}

	public async render(canvas: OffscreenCanvas) {
		const renderPosition = this.position()
			.subtract(this.size().multiply(this.origin()))
			.add(new Vector2(1920 / 2, 1080 / 2))
		const size = this.size()

		if (size.x === 0 || size.y === 0) return

		if (this.filters.length == 0) {
			//Render directly to canvas
			this.image.position = renderPosition
			this.image.size = size

			await this.image.render(canvas)
		} else {
			this.image.position = Vector2.zero()
			this.image.size = size

			const elementCanvas = new OffscreenCanvas(size.x, size.y)

			await this.image.render(elementCanvas)

			for (const filter of this.filters) {
				await filter.render(elementCanvas)
			}

			canvas
				.getContext('2d')
				.drawImage(elementCanvas, renderPosition.x, renderPosition.y, size.x, size.y)
		}
	}
}
