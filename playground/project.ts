clip(function* () {
	const rect = add(new Rect(new Vector2(0, 0), new Vector2(200, 200)))

	while (true) {
		yield null

		rect.position.x += 4
	}
})
