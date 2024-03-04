clip(function* () {
	const rect = add(new Rect(new Vector2(0, 0), new Vector2(200, 200), new Vector4(1, 0, 0, 1)))
	const rect2 = add(new Rect(new Vector2(0, 0), new Vector2(-200, -200), new Vector4(0, 1, 1, 1)))

	while (true) {
		yield null

		rect.position.x += -5
		rect.position.y += -5
	}
})
