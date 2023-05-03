// import testImage from '../Assets/image.png'
// import testImage2 from '../Assets/test.png'
import video from '../Assets/test2.mp4'

import {
	Ease,
	EaseIn,
	EaseOut,
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
	async function* ({ add, relative, wait, waitForMarker, aside, remove }: SceneContext) {
		const videoEl = add(
			new VectorVideo({
				position: relative(new Vector(0.5, 0.5)),
				size: new Vector(1920, 1080),
				video: video,
			})
		)

		aside(videoEl.play())

		yield* waitForMarker('open security')

		const background = add(
			new Rect({
				size: new Vector(0, 0),
				position: new Vector(203, 686),
				radius: 40,
				color: color('#141414'),
			})
		)

		const zoomText = add(
			new VectorText({
				text: 'DH',
				position: new Vector(203, 686),
				size: 13,
				color: color('#ffffff'),
				origin: new Vector(0.5, 0.5),
			})
		)

		background.size(new Vector(1920, 1080), 1, EaseIn)
		background.position(relative(new Vector(0.5, 0.5)), 1, EaseIn)
		background.radius(0, 1, EaseIn)

		yield* wait(0.2)

		zoomText.size(200, 2, Ease)
		yield* zoomText.position(relative(new Vector(0.5, 0.5)), 2, Ease)

		remove(zoomText)

		const D = add(
			new VectorText({
				text: 'D',
				position: relative(new Vector(0.46, 0.5)),
				size: 200,
				color: color('#ffffff'),
				origin: new Vector(0.5, 0.5),
			})
		)

		const H = add(
			new VectorText({
				text: 'H',
				position: relative(new Vector(0.54, 0.5)),
				size: 200,
				color: color('#ffffff'),
				origin: new Vector(0.5, 0.5),
			})
		)

		D.position(relative(new Vector(0.23, 0.5)), 1, Ease)
		H.position(relative(new Vector(0.5, 0.5)), 1, Ease)

		function* animateText(text: string, position: Vector) {
			let offset = 0
			for (const letter of text) {
				let width = 40

				switch (letter) {
					case 'f':
						width = 70
						break
					case 'e':
						width = 90
						break
					case 'm':
						width = 160
						break
					case 'a':
						width = 100
						break
				}

				const letterEl = add(
					new VectorText({
						text: letter,
						position: position.add(new Vector(offset)).subtract(new Vector(0, 70 + 20)),
						size: 180,
						color: color('#ffffff00'),
						origin: new Vector(0, 0),
					})
				)

				offset += width

				letterEl.position(letterEl.position().add(new Vector(0, 20)), 1, EaseOut)
				letterEl.color(color('#ffffff'), 1, EaseOut)

				yield* wait(0.2)
			}
		}

		yield* wait(1)

		aside(animateText('iffie', D.position().add(relative(new Vector(0.032)))))
		aside(animateText('ellman', H.position().add(relative(new Vector(0.033)))))
	}
)
