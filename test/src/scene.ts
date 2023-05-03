// import testImage from '../Assets/image.png'
// import testImage2 from '../Assets/test.png'
import video from '../Assets/test.mp4'

import {
	Ease,
	Ellipse,
	Linear,
	Rect,
	SceneContext,
	Vector,
	VectorImage,
	VectorText,
	VectorVideo,
	color,
	makeScene,
	reactive,
} from '@vector-engine/core'

export default makeScene(
	'Scene',
	async function* ({ add, relative, wait, waitForMarker }: SceneContext) {
		const videoEl = add(
			new VectorVideo({
				position: relative(new Vector(0.5, 0.5)),
				size: new Vector(1920, 1080),
				video: video,
			})
		)

		yield* videoEl.play()

		yield* wait(0.2)

		yield* waitForMarker('Hi')
		yield* waitForMarker('Bye')

		const pointA = add(
			new Rect({
				position: relative(new Vector(0.2, 0.5)),
				size: new Vector(5, 200),
			})
		)

		const pointB = add(
			new Rect({
				position: relative(new Vector(0.8, 0.5)),
				size: new Vector(5, 200),
			})
		)

		const test = add(
			new Ellipse({
				position: relative(new Vector(0.2, 0.5)),
				size: new Vector(50, 50),
			})
		)

		// yield* wait(1)

		// yield* test.position(relative(new Vector(0.8, 0.5)), 1, Linear)

		// console.log(test.position().equals(relative(new Vector(0.8, 0.5))))
	}
)
