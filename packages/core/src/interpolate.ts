import { Vector2, Vector3, Vector4 } from './vector'

export type Interpolator<T> = (a: T, b: T, t: number) => T

export function interpolate(a: Vector4, b: Vector4, t: number): Vector4
export function interpolate(a: Vector3, b: Vector3, t: number): Vector3
export function interpolate(a: Vector2, b: Vector2, t: number): Vector2
export function interpolate(a: number, b: number, t: number): number
export function interpolate<T>(a: T, b: T, t: number): T
export function interpolate<T>(a: T, b: T, t: number): T {
	if (a instanceof Vector4 && b instanceof Vector4)
		return <T>(
			new Vector4(
				a.x + (b.x - a.x) * t,
				a.y + (b.y - a.y) * t,
				a.z + (b.z - a.z) * t,
				a.w + (b.w - a.w) * t
			)
		)

	if (a instanceof Vector3 && b instanceof Vector3)
		return <T>new Vector3(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, a.z + (b.z - a.z) * t)

	if (a instanceof Vector2 && b instanceof Vector2)
		return <T>new Vector2(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)

	if (typeof a === 'number' && typeof b === 'number') return <T>(a + (b - a) * t)

	return a
}

export type TimingFunction = (t: number) => number
