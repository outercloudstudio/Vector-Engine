clip(function* () {
	const interLowerAtals = new FontAtlas('InterLower.png', 64, 'abcdefghijklmnopqrstuvwxyz', 5)
	const inter = new Font([interLowerAtals], 5)

	const background = add(
		new Rect({
			size: new Vector2(1920, 1080),
			color: new Vector4(9 / 256, 10 / 256, 20 / 256, 1),
			order: -100,
		})
	)

	const rect = add(
		new Rect({
			size: new Vector2(400, 400),
			color: new Vector4(0.5, 0.5, 0, 1),
			position: new Vector2(100, 100),
			rotation: 0,
			radius: 100,
		})
	)

	yield rect.position.to(new Vector2(0, 0), 3, linear)
	yield rect.rotation.to(6, 3, ease)

	yield* rect.color.to(new Vector4(0, 0.5, 0.5, 1), 1, linear)
	yield* rect.color.to(new Vector4(0.5, 0, 0.5, 1), 1, linear)
	yield* rect.color.to(new Vector4(0.5, 0.5, 0, 1), 1, linear)
})
