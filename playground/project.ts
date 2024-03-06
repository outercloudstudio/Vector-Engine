clip(function* () {
	const background = add(
		new Rect({
			position: new Vector2(-1920 / 2, -1080 / 2),
			size: new Vector2(1920, 1080),
			color: new Vector4(12 / 256, 12 / 256, 12 / 256, 1),
		})
	)

	function easeInOutCubic(x: number): number {
		return x < 0.5 ? 4 * x * x * x : 1 - Math.pow(-2 * x + 2, 3) / 2
	}

	let progress = 0
	let direction = 1

	const rect = add(
		new Rect({
			position: new Vector2(50, -200),
			size: new Vector2(400, 400),
			color: new Vector4(219 / 255, 30 / 255, 101 / 255, 1),
			radius: () => easeInOutCubic(progress) * 200,
		})
	)

	const ellipse = add(
		new Ellipse({
			position: () => new Vector2(-400, -50 + easeInOutCubic(progress) * 200 - 100),
			size: new Vector2(100, 100),
			color: new Vector4(19 / 256, 173 / 256, 235 / 256, 1),
		})
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
