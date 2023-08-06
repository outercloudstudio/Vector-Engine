import {
	DistortionFilter,
	Filter,
	GrayScaleFilter,
	Scene,
	Vector2,
	VectorImage,
	reactive,
	scene,
} from '@vector-engine/core'
import image from '../image.png'

export default scene(function* (scene: Scene) {
	const img = scene.add(
		new VectorImage({
			image,
			position: new Vector2(-400, 0),
			size: new Vector2(500, 400),
		})
	)
	const distortionFilter = img.filter(new DistortionFilter({ distortion: -0.3 }))
	distortionFilter.distortion(0, 1, t => t)

	const img2 = scene.add(
		new VectorImage({
			image,
			position: new Vector2(400, 0),
			size: new Vector2(500, 400),
		})
	)

	// img.size(new Vector2(img.size().x, 0), 1, t => t)
	img2.size(reactive(() => new Vector2(img.size().y, img.size().x)))
})
