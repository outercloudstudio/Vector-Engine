clip(function* () {
	const textSizes = [
		new Vector2(102, 46),
		new Vector2(285, 46),
		new Vector2(230, 58),
		new Vector2(15, 43),
	]

	const background = add(
		new Rect({
			size: new Vector2(1920, 1080),
			color: new Vector4(9 / 256, 10 / 256, 20 / 256, 1),
			order: -100,
		})
	)

	function* animateText(text: Clip) {
		const originalSize = text.size.value
		text.size = react(new Vector2(0, 0))

		yield text.size.to(originalSize, 0.5, ease)
	}

	let x = -700
	let y = 200

	for (let index = 1; index <= textSizes.length; index++) {
		const textSize = textSizes[index - 1]

		let textNumberText = index.toString()
		if (textNumberText.length === 1) textNumberText = '0' + textNumberText

		let yOffset = 0

		if (textSize.y == 58) yOffset = -12
		if (textSize.y == 43) yOffset = -2

		const text = add(
			new Clip({
				clip: `quote/quote_Text${textNumberText}.png`,
				position: new Vector2(x, y + yOffset),
				origin: new Vector2(0, 0),
				size: textSize,
				order: 1,
			})
		)

		yield animateText(text)

		yield* seconds(0.2)

		x += textSize.x + 20
	}
})
