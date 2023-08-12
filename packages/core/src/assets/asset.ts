export class Asset {
	constructor() {}

	toFrame(frame: number) {}

	async render(canvas: OffscreenCanvas) {}
	async renderAudio(frame: number, length: number) {}
	async previewAudio(frame: number) {}
}
