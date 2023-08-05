export class Vector2 {
	x: number
	y: number

	constructor(x: number, y: number) {
		this.x = x
		this.y = y
	}

	public static zero() {
		return new Vector2(0, 0)
	}

	public static one() {
		return new Vector2(1, 1)
	}

	public static half() {
		return new Vector2(0.5, 0.5)
	}

	public add(value: number): Vector2
	public add(value: Vector2): Vector2
	public add(value: number | Vector2): Vector2 {
		if (value instanceof Vector2) {
			return new Vector2(this.x + value.x, this.y + value.y)
		}

		return new Vector2(this.x + value, this.y + value)
	}

	public subtract(value: number): Vector2
	public subtract(value: Vector2): Vector2
	public subtract(value: number | Vector2): Vector2 {
		if (value instanceof Vector2) {
			return new Vector2(this.x - value.x, this.y - value.y)
		}

		return new Vector2(this.x - value, this.y - value)
	}

	public multiply(value: number): Vector2
	public multiply(value: Vector2): Vector2
	public multiply(value: number | Vector2): Vector2 {
		if (value instanceof Vector2) {
			return new Vector2(this.x * value.x, this.y * value.y)
		}

		return new Vector2(this.x * value, this.y * value)
	}

	public divide(value: number): Vector2
	public divide(value: Vector2): Vector2
	public divide(value: number | Vector2): Vector2 {
		if (value instanceof Vector2) {
			return new Vector2(this.x / value.x, this.y / value.y)
		}

		return new Vector2(this.x / value, this.y / value)
	}
}

export class Bounds {
	public start: Vector2
	public size: Vector2

	constructor(start: Vector2, size: Vector2) {
		this.start = start
		this.size = size
	}
}
