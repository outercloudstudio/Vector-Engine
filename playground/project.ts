clip(function* () {
	add(
		new Rect({
			position: new Vector2(-1920 / 2, -1080 / 2),
			size: new Vector2(1920, 1080),
			color: new Vector4(0, 0, 1, 0.2),
		})
	)

	const clip = add(
		new Clip({
			clip: 'test.ts',
			position: new Vector2(-500, -300),
			size: new Vector2(1000, 600),
		})
	)

	let progress = react(0)

	const rect = add(
		new Rect({
			position: new Vector2(-800, -200),
			size: new Vector2(400, 400),
			color: new Vector4(219 / 255, 30 / 255, 101 / 255, 1),
			radius: () => progress.value * 200,
		})
	)

	yield progress.bounce(1, 1, ease)

	yield* clip.frame.to(120, 2)

	// const background = add(
	// 	new Rect({
	// 		position: new Vector2(-1920 / 2, -1080 / 2),
	// 		size: new Vector2(1920, 1080),
	// 		color: new Vector4(12 / 256, 12 / 256, 12 / 256, 1),
	// 	})
	// )

	// let progress = react(0)
	// let progress2 = react(0)

	// const rect = add(
	// 	new Rect({
	// 		position: new Vector2(50, -200),
	// 		size: new Vector2(400, 400),
	// 		color: new Vector4(219 / 255, 30 / 255, 101 / 255, 1),
	// 		radius: () => progress.value * 200,
	// 	})
	// )

	// const ellipse = add(
	// 	new Ellipse({
	// 		position: () => new Vector2(-400, -50 + progress.value * 200 - 100),
	// 		size: new Vector2(100, 100),
	// 		color: new Vector4(19 / 256, 173 / 256, 235 / 256, 1),
	// 	})
	// )

	// const rect2 = add(
	// 	new Rect({
	// 		position: new Vector2(-600, -450),
	// 		size: () => new Vector2(400 + progress2.value * 700, 100),
	// 		color: new Vector4(30 / 255, 219 / 255, 101 / 255, 1),
	// 	})
	// )

	// const ellipse2Background = add(
	// 	new Ellipse({
	// 		position: new Vector2(-150 - 5, 200 - 5),
	// 		size: new Vector2(110, 110),
	// 		color: () => new Vector4(1 - progress2.value, 1 - progress2.value, 1 - progress2.value, 1),
	// 	})
	// )

	// const ellipse2 = add(
	// 	new Ellipse({
	// 		position: new Vector2(-150, 200),
	// 		size: new Vector2(100, 100),
	// 		color: () => new Vector4(progress2.value, progress2.value, progress2.value, 1),
	// 	})
	// )

	// yield progress.bounce(1, 1, ease)

	// yield* seconds(0.5)

	// yield progress2.bounce(1, 1, ease, 2)
})
