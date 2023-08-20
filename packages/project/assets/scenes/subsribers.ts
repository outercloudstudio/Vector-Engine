import {
	type Scene,
	scene,
	VectorText,
	reactive,
	Vector2,
	animatedReactiveProperty,
} from '@vector-engine/core'
import subscribers from './subs'
import { interpolate } from '@vector-engine/core/src/interpolate'

export default scene(function* (scene: Scene) {
	const position = animatedReactiveProperty(
		reactive(new Vector2(-1080 + 180, 1080)),
		interpolate
	).bind({ scene })

	let offset = 0

	let row = 0

	for (const subscriber of subscribers) {
		let localOffset = offset
		let localRow = row

		scene.add(
			new VectorText({
				text: subscriber,
				size: 30,
				position: reactive(() => position().add(new Vector2(localRow * 350, localOffset))),
			})
		)

		row++

		row = row % 3

		if (row == 0) offset += 40
	}

	position(new Vector2(-1080 + 180, -offset * 2), 15, t => t)
})
