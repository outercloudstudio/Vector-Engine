function createRandomWithSeed(seed: number): () => number {
	return () => {
		seed |= 0
		seed = (seed + 0x6d2b79f5) | 0
		let imul = Math.imul(seed ^ (seed >>> 15), 1 | seed)
		imul = (imul + Math.imul(imul ^ (imul >>> 7), 61 | imul)) ^ imul
		return ((imul ^ (imul >>> 14)) >>> 0) / 4294967296
	}
}

function* createStars() {
	for (let i = 0; i < 1; i++) {
		yield createStar(i + 200)
		yield* seconds(0.02)
	}
}

function* createStar(seed: number) {
	const random = createRandomWithSeed(seed)

	const colors = [
		new Vector4(166 / 255, 200 / 255, 255 / 255, 1),
		new Vector4(255 / 255, 231 / 255, 166 / 255, 1),
		new Vector4(166 / 255, 255 / 255, 188 / 255, 1),
		new Vector4(1, 1, 1, 1),
	]

	let direction = random() * Math.PI * 2

	const star = add(
		new Ellipse({
			position: new Vector2(Math.cos(direction) * 1200, Math.sin(direction) * 1200),
			size: new Vector2(50, 50),
			color: colors[Math.floor(random() * 4)],
		})
	)

	const speed = 0.9 + random() * 0.2

	yield star.size.to(new Vector2(0, 0), speed)
	yield* star.position.to(new Vector2(0, 0), speed)

	remove(star)
}

clip(function* () {
	// add(
	// 	new Ellipse({
	// 		color: new Vector4(166 / 255, 200 / 255, 255 / 255, 1),
	// 	})
	// )
	// add(
	// 	new Ellipse({
	// 		position: new Vector2(200, 0),
	// 		color: new Vector4(255 / 255, 231 / 255, 166 / 255, 1),
	// 	})
	// )

	// const background = add(
	// 	new Rect({
	// 		size: new Vector2(1920, 1080),
	// 		color: new Vector4(9 / 256, 10 / 256, 20 / 256, 1),
	// 		order: -100,
	// 	})
	// )
	// const clip = add(
	// 	new Rect({
	// 		// clip: 'test.ts',
	// 		size: new Vector2(20, 20),
	// 		order: 1,
	// 		rotation: -Math.PI / 20,
	// 	})
	// )
	yield createStars()
	// yield* seconds(1)
	// yield clip.size.to(new Vector2(1000, 700), 1, ease)
	// yield clip.rotation.bounce(Math.PI / 20, 2, ease)
})
