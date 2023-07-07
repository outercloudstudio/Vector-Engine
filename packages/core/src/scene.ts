export async function scene(generator: () => AsyncGenerator): Promise<Scene> {
	const scene = new Scene()

	await scene.load(generator)

	return scene
}

export class Scene {
	private context: AsyncGenerator

	async load(generator: () => AsyncGenerator) {
		this.context = generator()
	}
}
