clip(function* () {
	const background = add(
		new Rect(
			new Vector2(-1920 / 2, -1080 / 2),
			new Vector2(1920, 1080),
			new Vector4(12 / 256, 12 / 256, 12 / 256, 1),
			0
		)
	)

	function easeInOutCubic(x: number): number {
		return x < 0.5 ? 4 * x * x * x : 1 - Math.pow(-2 * x + 2, 3) / 2
	}

	let progress = 0
	let direction = 1

	const rect = add(
		new Rect(
			new Vector2(50, -200),
			new Vector2(400, 400),
			new Vector4(219 / 255, 30 / 255, 101 / 255, 1),
			() => easeInOutCubic(progress) * 200
		)
	)

	const ellipse = add(
		new Ellipse(
			() => new Vector2(-400, -50 + easeInOutCubic(progress) * 200 - 100),
			new Vector2(100, 100),
			new Vector4(19 / 256, 173 / 256, 235 / 256, 1)
		)
	)

	while (true) {
		yield null

		progress += (1 / 60) * direction

		if (progress >= 1) {
			direction = -1
			progress = 1
		}

		if (progress <= 0) {
			direction = 1
			progress = 0
		}
	}
})
