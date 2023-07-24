import { Rect, Scene, Vector2, VectorImage, scene } from '@vector-engine/core'
import { reactive } from '@vector-engine/core/src/reactive'
import image from '../image.png'

export default scene(function* (scene: Scene) {
	const rect = scene.add(new Rect(Vector2.zero(), new Vector2(100, 100)))
	const rect2 = scene.add(new Rect(new Vector2(100, 0), new Vector2(100, 100)))
	rect2.size(reactive(() => rect.size()))

	rect.size(Vector2.zero(), 1, t => t)

	scene.add(new VectorImage(image, Vector2.zero(), new Vector2(400, 400)))
})
