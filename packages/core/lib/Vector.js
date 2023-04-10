export class Vector {
    x;
    y;
    z;
    w;
    constructor(x, y, z, w) {
        this.x = x || 0;
        this.y = y || 0;
        this.z = z || 0;
        this.w = w || 0;
    }
    add(a) {
        return new Vector(this.x + a.x, this.y + a.y, this.z + a.z, this.w + a.w);
    }
    subtract(a) {
        return new Vector(this.x - a.x, this.y - a.y, this.z - a.z, this.w - a.w);
    }
    multiply(a) {
        return new Vector(this.x * a.x, this.y * a.y, this.z * a.z, this.w * a.w);
    }
    divide(a) {
        return new Vector(this.x / a.x, this.y / a.y, this.z / a.z, this.w / a.w);
    }
    lerp(a, t) {
        return new Vector(this.x + (a.x - this.x) * t, this.y + (a.y - this.y) * t, this.z + (a.z - this.z) * t, this.w + (a.w - this.w) * t);
    }
    clone() {
        return new Vector(this.x, this.y, this.z, this.w);
    }
    dot(a) {
        return this.x * a.x + this.y * a.y + this.z * a.z + this.w * a.w;
    }
    magnitude() {
        return Math.sqrt(this.x * this.x + this.y * this.y + this.z * this.z + this.w * this.w);
    }
    equals(a) {
        return this.x == a.x && this.y == a.y && this.z == a.z && this.w == a.w;
    }
}
//# sourceMappingURL=Vector.js.map