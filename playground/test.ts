clip(function* () {
	const background = add(
		new Rect({
			size: new Vector2(1920, 1080),
		})
	)

	let progress = react(0)

	const rect = add(
		new Rect({
			size: new Vector2(400, 400),
			color: new Vector4(219 / 255, 30 / 255, 101 / 255, 1),
			radius: () => progress.value * 200,
		})
	)

	yield progress.bounce(1, 1, ease)
})
