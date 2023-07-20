import { Rect, Scene, Vector2, VectorImage, scene } from '@vector-engine/core'
import testImage from '../BQvkeq.png'

export default scene(function* (scene: Scene) {
	scene.add(new Rect(Vector2.zero(), new Vector2(100, 100)))

	for (let frame = 0; frame < 30; frame++) {
		yield 1
	}

	scene.add(new VectorImage(testImage, Vector2.zero(), new Vector2(400, 400)))
	scene.add(new Rect(Vector2.zero(), new Vector2(100, 100)))
})
