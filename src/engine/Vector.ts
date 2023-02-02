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

  add(a: Vector): Vector {
    return new Vector(this.x + a.x, this.y + a.y, this.z + a.z, this.w + a.w)
  }

  subtract(a: Vector): Vector {
    return new Vector(this.x - a.x, this.y - a.y, this.z - a.z, this.w - a.w)
  }

  multiply(a: Vector): Vector {
    return new Vector(this.x * a.x, this.y * a.y, this.z * a.z, this.w * a.w)
  }

  divide(a: Vector): Vector {
    return new Vector(this.x / a.x, this.y / a.y, this.z / a.z, this.w / a.w)
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
    return Math.sqrt(
      this.x * this.x + this.y * this.y + this.z * this.z + this.w * this.w
    )
  }
}

export class ReactiveVector extends Vector {
  _x: number = 0
  _y: number = 0
  _z: number = 0
  _w: number = 0

  reactor: (vector: Vector, previousVector: Vector) => void

  constructor(
    reactor: (vector: Vector, previousVector: Vector) => void,
    x?: number,
    y?: number,
    z?: number,
    w?: number
  ) {
    super(x, y, z, w)

    this.reactor = reactor

    this._x = this.x
    Object.defineProperty(this, 'x', {
      get: () => this._x,
      set: (value: number) => {
        const original = new Vector(this._x, this._y, this._z, this._w)

        this._x = value

        reactor(this, original)
      },
    })

    this._y = this.y
    Object.defineProperty(this, 'y', {
      get: () => this._y,
      set: (value: number) => {
        const original = new Vector(this._x, this._y, this._z, this._w)

        this._y = value

        reactor(this, original)
      },
    })

    this._z = this.z
    Object.defineProperty(this, 'z', {
      get: () => this._z,
      set: (value: number) => {
        const original = new Vector(this._x, this._y, this._z, this._w)

        this._z = value

        reactor(this, original)
      },
    })

    this._w = this.w
    Object.defineProperty(this, 'w', {
      get: () => this._w,
      set: (value: number) => {
        const original = new Vector(this._x, this._y, this._z, this._w)

        this._w = value

        reactor(this, original)
      },
    })
  }
}
