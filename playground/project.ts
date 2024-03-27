import { mullishBoldItalic } from './fonts/mullish'

clip(function* () {
	const background = add(
		new Rect({
			color: new Vector4(0.05, 0.05, 0.05, 1),
			size: new Vector2(1920, 1080),
		})
	)

	let x = -220

	const text = 'Technique 1'

	for (const character of text) {
		const text = add(
			new VectText({
				font: mullishBoldItalic,
				text: character,
				character_size: 100,
				origin: new Vector2(0, 0.3),
				position: new Vector2(x, 0),
			})
		)

		x += text.size.value.x

		text.character_size.value = 0
		text.color.value = new Vector4(1, 1, 1, 0)
		yield text.character_size.to(100, 0.5, easeOutBack)
		yield text.color.to(new Vector4(1, 1, 1, 1), 0.5, ease)

		yield* seconds(0.05)
	}
})
