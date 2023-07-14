export type Meta = {
	assets: { [key: string]: AssetMeta }
}

export type AssetMeta = {
	name: string
	path: string
}
