import { Scene, Vector2, VectorImage, scene } from '@vector-engine/core'
import { reactive } from '@vector-engine/core/src/reactive'
import image from '../image.png'
import { Filter } from '@vector-engine/core/src/filter'

export default scene(function* (scene: Scene) {
	const img = scene.add(
		new VectorImage({
			image,
			position: new Vector2(-400, 0),
			size: new Vector2(500, 400),
		})
	)
	img.filter(new Filter())

	const img2 = scene.add(
		new VectorImage({
			image,
			position: new Vector2(400, 0),
			size: new Vector2(500, 400),
		})
	)

	img.size(new Vector2(img.size().x, 0), 1, t => t)
	img2.size(reactive(() => new Vector2(img.size().y, img.size().x)))
})
