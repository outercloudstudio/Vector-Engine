import { Asset } from '../core'

export type Clip = {
	id: string
	assetId: string
	frame: number
	firstClipFrame: number
	length: number
}

export type Project = {
	timeline: { [key: number]: Clip[] }
}
