declare module '*.png' {
	const src: () => import('@vector-engine/core').ImageAsset
	export default src
}

declare module '*.mp4' {
	const src: () => import('@vector-engine/core').VideoAsset
	export default src
}
