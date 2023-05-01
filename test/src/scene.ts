import testImage from '../Assets/image.png'
import testImage2 from '../Assets/test.png'
// import video from '../Assets/test.mp4'

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
	reactive,
} from '@vector-engine/core'

export default async function* ({ add, relative, aside }: SceneContext) {
	// const videoEl = add(
	//   new VectorVideo({
	//     position: relative(new Vector(0.5, 0.5)),
	//     size: new Vector(1920, 1080),
	//     video: video,
	//   })
	// )

	// aside(videoEl.play())

	const test = add(
		new Rect({
			position: relative(new Vector(0.2, 0.5)),
		})
	)

	const test2 = add(
		new Rect({
			position: reactive(() => test.position().add(new Vector(-300, 300))),
			// rotation: reactive(() => test.position().x),
			scale: reactive(() => new Vector(test.position().x / 1000, test.position().x / 1000)),
			// radius: reactive(() => test.position().x / 10),
		})
	)

	test2.rotation(360, 1, Ease)
	yield* test.position(relative(new Vector(0.8, 0.5)), 1, Ease)

	const dot = add(
		new Ellipse({
			position: relative(new Vector(0.2, 0.5)),
			size: new Vector(10, 10),
		})
	)

	const text = add(
		new VectorText({
			position: relative(new Vector(0.2, 0.5)),
			rotation: 45,
		})
	)

	const rotateAside = await aside(async function* () {
		while (true) {
			yield* await text.rotation(-45, 1, Ease)
			yield* await text.rotation(45, 1, Ease)
		}
	})

	const image = add(
		new VectorImage({
			image: testImage2,
			position: relative(new Vector(0.8, 0.5)),
			color: color('#00FF00'),
			origin: new Vector(0, 0),
			// rotation: 45,
		})
	)

	const dot2 = add(
		new Ellipse({
			position: relative(new Vector(0.8, 0.5)),
			size: new Vector(10, 10),
		})
	)
	yield* await image.rotation(360, 1, Linear)
	// yield* square.animatePosition(relative(new Vector(0.8, 0.5)), 1, Ease)
	// yield* square.animatePosition(relative(new Vector(0.2, 0.5)), 1, Ease)
}
