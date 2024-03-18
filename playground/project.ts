clip(function* () {
	const mullishBoldItalic = new FontAtlas(
		'fonts/Mullish Bold Italic.png',
		8,
		8,
		0.25,
		0.55,
		'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ. "-123',
		{
			w: 0.8,
			m: 0.74,
			i: 0.24,
			t: 0.35,
			j: 0.4,
			p: 0.6,
			r: 0.4,
			o: 0.5,
			e: 0.45,
			s: 0.45,
			c: 0.45,
			' ': 0.25,
			F: 0.38,
			A: 0.74,
			I: 0.3,
			L: 0.5,
		}
	)

	const background = add(
		new Rect({
			color: new Vector4(0.05, 0.05, 0.05, 1),
			size: new Vector2(1920, 1080),
		})
	)

	const flash = add(
		new Rect({
			color: new Vector4(1, 1, 1, 1),
			size: new Vector2(1920, 0),
		})
	)

	yield* flash.size.to(new Vector2(1920, 100), 0.2, ease)

	yield flash.size.to(new Vector2(1920, 1080), 0.1, ease)

	yield background.color.to(new Vector4(0.05, 0.08, 0.05, 1), 0.2, ease)

	yield* flash.color.to(new Vector4(1, 1, 1, 0), 0.4, ease)

	const number1 = add(
		new VectText({
			position: new Vector2(-600 - 100, 0 - 100),
			text: '1',
			font: mullishBoldItalic,
			size: 200,
			color: new Vector4(1, 1, 1, 0),
		})
	)

	yield* number1.color.to(new Vector4(1, 1, 1, 1), 0.2, ease)

	const number2 = add(
		new VectText({
			position: new Vector2(-100, 0 - 100),
			text: '2',
			font: mullishBoldItalic,
			size: 200,
			color: new Vector4(1, 1, 1, 0),
		})
	)

	yield* number2.color.to(new Vector4(1, 1, 1, 1), 0.2, ease)

	const number3 = add(
		new VectText({
			position: new Vector2(600 - 100, 0 - 100),
			text: '3',
			font: mullishBoldItalic,
			size: 200,
			color: new Vector4(1, 1, 1, 0),
		})
	)

	yield* number3.color.to(new Vector4(1, 1, 1, 1), 0.2, ease)

	yield* seconds(1)

	const outerWilds = add(
		new Clip({
			clip: 'OuterWilds.png',
			size: new Vector2(0, 0),
			position: new Vector2(-600, 0),
			color: new Vector4(1, 1, 1, 0),
		})
	)

	yield outerWilds.color.to(new Vector4(1, 1, 1, 1), 0.2, ease)
	yield number1.color.to(new Vector4(1, 1, 1, 0), 0.2, ease)
	yield* outerWilds.size.to(new Vector2(500, 500), 0.2, easeOutBack)

	const spelunky = add(
		new Clip({
			clip: 'Spelunky.png',
			size: new Vector2(0, 0),
			position: new Vector2(0, 0),
			color: new Vector4(1, 1, 1, 0),
		})
	)

	yield spelunky.color.to(new Vector4(1, 1, 1, 1), 0.2, ease)
	yield number2.color.to(new Vector4(1, 1, 1, 0), 0.2, ease)
	yield* spelunky.size.to(new Vector2(400, 500), 0.2, easeOutBack)

	const spore = add(
		new Clip({
			clip: 'Spore.png',
			size: new Vector2(0, 0),
			position: new Vector2(600, 0),
			color: new Vector4(1, 1, 1, 0),
		})
	)

	yield spore.color.to(new Vector4(1, 1, 1, 1), 0.2, ease)
	yield number3.color.to(new Vector4(1, 1, 1, 0), 0.2, ease)
	yield* spore.size.to(new Vector2(500, 500), 0.2, easeOutBack)
})
