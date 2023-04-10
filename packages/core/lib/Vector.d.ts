export declare class Vector {
    x: number;
    y: number;
    z: number;
    w: number;
    constructor(x?: number, y?: number, z?: number, w?: number);
    add(a: Vector): Vector;
    subtract(a: Vector): Vector;
    multiply(a: Vector): Vector;
    divide(a: Vector): Vector;
    lerp(a: Vector, t: number): Vector;
    clone(): Vector;
    dot(a: Vector): number;
    magnitude(): number;
    equals(a: Vector): boolean;
}
