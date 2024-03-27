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
		})
	)

	add(new Rect({ origin: new Vector2(0, 0.5), position: new Vector2(title.size.value.x, 0) }))
})
