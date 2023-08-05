import { Rect, Scene, Vector2, VectorImage, scene } from '@vector-engine/core'
import { reactive } from '@vector-engine/core/src/reactive'
import image from '../image.png'
import { Filter } from '@vector-engine/core/src/filter'

export default scene(function* (scene: Scene) {
	const rect = scene.add(new Rect(Vector2.zero(), new Vector2(100, 100)))
	const rect2 = scene.add(new Rect(new Vector2(100, 0), new Vector2(100, 100)))
	rect2.size(reactive(() => rect.size()))

	rect.size(Vector2.zero(), 1, t => t)

	const img = scene.add(
		new VectorImage({
			image,
			position: new Vector2(-400, -300),
			size: new Vector2(400, 400),
		})
	)

	img.filter(new Filter())

	img.size(reactive(() => new Vector2(rect.size().x * 2, rect.size().y * 2)))
})
