import { mullishBoldItalic } from './fonts/mullish'

clip(function* () {
	const background = add(
		new Rect({
			color: new Vector4(0.05, 0.05, 0.05, 1),
			size: new Vector2(1920, 1080),
		})
	)

	const title = add(
		new VectText({
			font: mullishBoldItalic,
			text: 'Technique 1',
			character_size: 100,
			origin: new Vector2(0.5, 0),
		})
	)

	add(new Rect({ size: new Vector2(1920, 8) }))
})
