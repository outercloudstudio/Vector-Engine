export class Vector2 {
	x: number
	y: number

	constructor(x: number, y: number) {
		this.x = x
		this.y = y
	}

	static zero() {
		return new Vector2(0, 0)
	}

	static one() {
		return new Vector2(1, 1)
	}

	static half() {
		return new Vector2(0.5, 0.5)
	}
}
