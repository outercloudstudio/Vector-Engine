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
			origin: new Vector2(0.5, 0.3),
		})
	)

	// add(
	// 	new VectText({
	// 		font: mullishBoldItalic,
	// 		text: 'Technique 1',
	// 		character_size: 100,
	// 		origin: new Vector2(0, 0),
	// 	})
	// )

	add(
		new Rect({
			origin: new Vector2(0, 0),
			position: new Vector2(
				title.size.value.x * title.origin.value.x,
				title.size.value.y * title.origin.value.y
			),
		})
	)

	// add(new Rect({ origin: new Vector2(0.5, 0.5), size: new Vector2(1920, 4) }))
})
