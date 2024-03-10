clip(function* () {
	const mullishBoldItalic = new FontAtlas(
		'fonts/Mullish Bold Italic.png',
		7,
		8,
		0.25,
		0.55,
		'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ. "-',
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
		new Clip({
			clip: 'Background.png',
			size: new Vector2(1920, 1080),
		})
	)

	const quoteLine1 = add(
		new VectText({
			position: new Vector2(-700, 200),
			text: 'The ambitious projects I had undertaken',
			font: mullishBoldItalic,
			size: 40,
		})
	)

	const quoteLine2A = add(
		new VectText({
			position: new Vector2(-700, 100),
			text: 'in the past',
			font: mullishBoldItalic,
			size: 40,
		})
	)

	const quoteFailed = add(
		new VectText({
			position: new Vector2(-335, 100),
			text: 'FAILED',
			font: mullishBoldItalic,
			size: 40,
			color: new Vector4(1.0, 0.0, 0.0, 1.0),
		})
	)

	const quoteLine2B = add(
		new VectText({
			position: new Vector2(-80, 100),
			text: 'because I had made the',
			font: mullishBoldItalic,
			size: 40,
		})
	)

	const quoteLine3 = add(
		new VectText({
			position: new Vector2(-700, 0),
			text: 'mistake of not proving out the core ideas',
			font: mullishBoldItalic,
			size: 40,
		})
	)

	const quoteLine4 = add(
		new VectText({
			position: new Vector2(-700, -100),
			text: 'in prototypes.',
			font: mullishBoldItalic,
			size: 40,
		})
	)

	const quoteA = add(
		new VectText({
			position: new Vector2(-800, 200),
			text: '"',
			font: mullishBoldItalic,
			size: 100,
		})
	)

	const quoteB = add(
		new VectText({
			position: new Vector2(640, -200),
			text: '"',
			font: mullishBoldItalic,
			size: 100,
		})
	)

	const attribution = add(
		new VectText({
			position: new Vector2(-700, -250),
			text: '- Chris Hecker - Developer of Spore',
			font: mullishBoldItalic,
			size: 30,
		})
	)
})
