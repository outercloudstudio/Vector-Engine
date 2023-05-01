export class Vector {
	x: number
	y: number
	z: number
	w: number

	constructor(x?: number, y?: number, z?: number, w?: number) {
		this.x = x || 0
		this.y = y || 0
		this.z = z || 0
		this.w = w || 0
	}

	add(a: Vector | number): Vector {
		if (a instanceof Vector) {
			return new Vector(this.x + a.x, this.y + a.y, this.z + a.z, this.w + a.w)
		} else {
			return new Vector(this.x + a, this.y + a, this.z + a, this.w + a)
		}
	}

	subtract(a: Vector): Vector {
		if (a instanceof Vector) {
			return new Vector(this.x - a.x, this.y - a.y, this.z - a.z, this.w - a.w)
		} else {
			return new Vector(this.x - a, this.y - a, this.z - a, this.w - a)
		}
	}

	multiply(a: Vector): Vector {
		if (a instanceof Vector) {
			return new Vector(this.x * a.x, this.y * a.y, this.z * a.z, this.w * a.w)
		} else {
			return new Vector(this.x * a, this.y * a, this.z * a, this.w * a)
		}
	}

	divide(a: Vector): Vector {
		if (a instanceof Vector) {
			return new Vector(this.x / a.x, this.y / a.y, this.z / a.z, this.w / a.w)
		} else {
			return new Vector(this.x / a, this.y / a, this.z / a, this.w / a)
		}
	}

	lerp(a: Vector, t: number): Vector {
		return new Vector(
			this.x + (a.x - this.x) * t,
			this.y + (a.y - this.y) * t,
			this.z + (a.z - this.z) * t,
			this.w + (a.w - this.w) * t
		)
	}

	clone(): Vector {
		return new Vector(this.x, this.y, this.z, this.w)
	}

	dot(a: Vector): number {
		return this.x * a.x + this.y * a.y + this.z * a.z + this.w * a.w
	}

	magnitude(): number {
		return Math.sqrt(this.x * this.x + this.y * this.y + this.z * this.z + this.w * this.w)
	}

	equals(a: Vector): boolean {
		return this.x == a.x && this.y == a.y && this.z == a.z && this.w == a.w
	}
}
