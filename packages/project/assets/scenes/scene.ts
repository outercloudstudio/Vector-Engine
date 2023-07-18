import { Rect, Scene, Vector2, scene } from '@vector-engine/core'

export default scene(function* (scene: Scene) {
	scene.add(new Rect(Vector2.zero(), new Vector2(100, 100)))

	for (let frame = 0; frame < 10; frame++) {
		yield 1
	}

	scene.add(new Rect(new Vector2(300, -300), new Vector2(100, 100)))
})
