export function EaseOutCircle(x: number): number {
	return Math.sqrt(1 - Math.pow(x - 1, 2))
}
