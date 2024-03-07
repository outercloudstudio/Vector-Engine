clip(function* () {
	const background = add(
		new Rect({
			position: new Vector2(-1920 / 2, -1080 / 2),
			size: new Vector2(1920, 1080),
			color: new Vector4(12 / 256, 12 / 256, 12 / 256, 1),
		})
	)

	let progress = react(0)

	const rect = add(
		new Rect({
			position: new Vector2(50, -200),
			size: new Vector2(400, 400),
			color: new Vector4(219 / 255, 30 / 255, 101 / 255, 1),
			radius: () => progress.value * 200,
		})
	)

	const ellipse = add(
		new Ellipse({
			position: () => new Vector2(-400, -50 + progress.value * 200 - 100),
			size: new Vector2(100, 100),
			color: new Vector4(19 / 256, 173 / 256, 235 / 256, 1),
		})
	)

	const rect2 = add(
		new Rect({
			position: new Vector2(-600, -450),
			size: () => new Vector2(400 + progress.value * 700, 100),
			color: new Vector4(30 / 255, 219 / 255, 101 / 255, 1),
		})
	)

	const ellipse2Background = add(
		new Ellipse({
			position: new Vector2(-150 - 5, 200 - 5),
			size: new Vector2(110, 110),
			color: () => new Vector4(1 - progress.value, 1 - progress.value, 1 - progress.value, 1),
		})
	)

	const ellipse2 = add(
		new Ellipse({
			position: new Vector2(-150, 200),
			size: new Vector2(100, 100),
			color: () => new Vector4(progress.value, progress.value, progress.value, 1),
		})
	)

	yield* progress.bounce(1, 1, ease)
})
