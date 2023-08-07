import {
	ColorFadeFilter,
	DistortionFilter,
	EaseOutCircle,
	Filter,
	GrayScaleFilter,
	Scene,
	Vector2,
	VectorImage,
	reactive,
	scene,
} from '@vector-engine/core'
import backgroundImage from '../Intro/Background1.png'
import psCharacterImage from '../Intro/PsCharacter.png'
import killTextBaseImage from '../Intro/KillTextBase.png'
import killTextTopImage from '../Intro/KillTextTop.png'
import darlingsTextBaseImage from '../Intro/DarlingsTextBase.png'
import darlingsTextTopImage from '../Intro/DarlingsTextTop.png'

export default scene(function* (scene: Scene) {
	const background = scene.add(
		new VectorImage({
			image: backgroundImage,
			size: new Vector2(1920, 1080),
		})
	)

	const killTextBase = scene.add(
		new VectorImage({
			image: killTextBaseImage,
			size: new Vector2(1920, 1080),
		})
	)

	const darlingsTextBase = scene.add(
		new VectorImage({
			image: darlingsTextBaseImage,
			size: new Vector2(1920, 1080),
		})
	)

	const psCharacter = scene.add(
		new VectorImage({
			image: psCharacterImage,
			size: new Vector2(1920, 1080),
		})
	)

	const killTextTop = scene.add(
		new VectorImage({
			image: killTextTopImage,
			size: reactive(() => killTextBase.size()),
		})
	)

	const darlingsTextTop = scene.add(
		new VectorImage({
			image: darlingsTextTopImage,
			size: reactive(() => darlingsTextBase.size()),
		})
	)

	killTextBase.size(killTextBase.size().multiply(1.05), 2, t => t)
	darlingsTextBase.size(darlingsTextBase.size().multiply(1.05), 2, t => t)
	psCharacter.size(psCharacter.size().multiply(1.1), 2, t => t)

	const distortionFilter = scene.add(new DistortionFilter({ distortion: -0.3 }))

	const flashFilter = scene.add(new ColorFadeFilter({ factor: 1 }))
	flashFilter.factor(0, 0.1, EaseOutCircle)

	yield* distortionFilter.distortion(0.01, 1, EaseOutCircle)

	distortionFilter.distortion(0.015, 1, t => t)
})
