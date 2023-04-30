export function getBestFittingBounds(width: number, height: number, aspect: number): { width: number; height: number } {
	const potentialWidthForHeight = height * aspect

	const bestWidth = Math.min(width, potentialWidthForHeight)
	const bestHeight = bestWidth / aspect

	return {
		width: bestWidth,
		height: bestHeight,
	}
}
