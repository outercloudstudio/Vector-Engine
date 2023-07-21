import { Vector2 } from './vector'

export type Interpolator<T> = (a: T, b: T, t: number) => T

export function interpolate(a: Vector2, b: Vector2, t: number): Vector2
export function interpolate(a: number, b: number, t: number): number
export function interpolate<T>(a: T, b: T, t: number): T
export function interpolate<T>(a: T, b: T, t: number): T {
	if (a instanceof Vector2 && b instanceof Vector2)
		return <T>new Vector2(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)

	if (typeof a === 'number' && typeof b === 'number') return <T>(a + (b - a) * t)

	return a
}

export type TimingFunction = (t: number) => number
