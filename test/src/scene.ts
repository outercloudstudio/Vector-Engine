import computerImage from '../Assets/computer.png'
import eyeImage from '../Assets/eye.png'

import {
	Ease,
	EaseIn,
	EaseOut,
	Ellipse,
	Linear,
	Rect,
	SceneContext,
	Vector,
	VectorImage,
	VectorText,
	VectorVideo,
	color,
	makeScene,
	reactive,
} from '@vector-engine/core'

function* shake(element: VectorImage | VectorText, size?: number) {
	for (let i = 0; i < 2; i++) {
		yield* element.rotation(size ? size : 3, 0.1, EaseOut)
		yield* element.rotation(size ? -size : -3, 0.1, EaseOut)
	}

	yield* element.rotation(0, 0.1, EaseOut)
}

export default makeScene(
	'Scene',
	async function* ({ add, relative, wait, waitForMarker, aside, remove, animate }: SceneContext) {
		const background = add(
			new Rect({
				position: relative(new Vector(0, 1)),
				size: new Vector(1920, 1080),
				color: color('#141414'),
			})
		)

		background.position(new Vector(), 0.5, Ease)

		const D = add(
			new VectorText({
				text: 'D',
				position: relative(new Vector(-0.27)),
				size: 200,
				color: color('#eeeeee00'),
			})
		)

		const H = add(
			new VectorText({
				text: 'H',
				size: 200,
				color: color('#eeeeee00'),
			})
		)

		D.color(color('#eeeeee'), 1, Ease)
		H.color(color('#eeeeee'), 1, Ease)

		const letterAlpha = reactive(1)

		function* animateText(text: string, position: Vector) {
			let offset = 0
			for (const letter of text) {
				let width = 40

				switch (letter) {
					case 'f':
						width = 70
						break
					case 'e':
						width = 90
						break
					case 'm':
						width = 160
						break
					case 'a':
						width = 100
						break
				}

				const letterEl = add(
					new VectorText({
						text: letter,
						position: position.add(new Vector(offset)).subtract(new Vector(0, 70 + 20)),
						size: 180,
						color: color('#eeeeee00'),
						origin: new Vector(0, 0),
					})
				)

				offset += width

				aside(function* () {
					letterEl.position(letterEl.position().add(new Vector(0, 20)), 0.5, EaseOut)
					yield* letterEl.color(color('#eeeeee'), 0.5, EaseOut)

					letterEl.color(reactive(() => new Vector(1, 1, 1, letterAlpha())))
				})

				yield* wait(0.06)
			}
		}

		yield* wait(1.5)

		yield* animateText('iffie', D.position().add(relative(new Vector(0.032))))
		aside(animateText('ellman', H.position().add(relative(new Vector(0.033)))))

		const KeyExchangeText = add(
			new VectorText({
				text: 'Key Exchange',
				position: new Vector(0, -150),
				size: 100,
				color: color('#eeeeee00'),
			})
		)

		KeyExchangeText.color(color('#aaaaaa'), 0.5, Ease)

		yield* waitForMarker('Intro')

		D.color(reactive(() => new Vector(1, 1, 1, letterAlpha())))
		H.color(reactive(() => new Vector(1, 1, 1, letterAlpha())))

		KeyExchangeText.color(color('#aaaaaa00'), 0.7, Ease)
		yield* animate(0.7, Ease, (time: number) => {
			letterAlpha(1 - time)
		})

		const SymetricEncryptionText = add(
			new VectorText({
				text: 'Symetric Encryption',
				position: new Vector(0, 400),
				size: 100,
				color: color('#eeeeee00'),
			})
		)
		SymetricEncryptionText.color(color('#eeeeee'), 0.5, Ease)

		yield* wait(2)

		const computer1 = add(
			new VectorImage({
				image: computerImage,
				position: new Vector(-500, -100),
				size: new Vector(200, 200),
				color: color('#eeeeee00'),
			})
		)

		computer1.color(color('#eeeeee'), 0.5, Ease)

		const message = add(
			new VectorText({
				text: 'Oranges!',
				position: new Vector(-800, 100),
				color: color('#eb773400'),
				size: 50,
			})
		)
		message.color(color('#eb7734'), 0.5, Ease)

		const secretKey = add(
			new VectorText({
				text: (Math.random() + 1).toString(36).substring(4),
				position: new Vector(-500, 100),
				color: color('#eeeeee00'),
				size: 50,
			})
		)
		secretKey.color(color('#eeeeee'), 0.5, Ease)

		yield* wait(1)

		message.position(new Vector(-200, 100), 0.8, Ease)
		yield* wait(0.4)
		message.text((Math.random() + 1).toString(36).substring(4))

		yield* wait(0.5)

		const computer2 = add(
			new VectorImage({
				image: computerImage,
				position: new Vector(500, -100),
				size: new Vector(200, 200),
				color: color('#eeeeee00'),
			})
		)

		computer2.color(color('#eeeeee'), 0.5, Ease)

		const secretKey2 = add(
			new VectorText({
				text: secretKey.text(),
				position: new Vector(500, 100),
				color: color('#eeeeee00'),
				size: 50,
			})
		)
		secretKey2.color(color('#eeeeee'), 0.5, Ease)

		yield* wait(0.5)

		message.position(new Vector(200, 100), 0.8, Ease)

		yield* wait(2.3)

		message.position(new Vector(800, 100), 0.8, Ease)
		yield* wait(0.4)
		message.text('Oranges!')

		yield* wait(0.7)

		message.color(color('#eb773400'), 0.5, Ease)

		secretKey.size(90, 0.5, Ease)
		secretKey2.size(90, 0.5, Ease)

		yield* wait(1)

		secretKey2.color(color('#aaaaaa00'), 0.5, Ease)
		secretKey.position(new Vector(0, 100), 0.5, Ease)

		yield* wait(5)

		const hacker = add(
			new VectorImage({
				image: eyeImage,
				position: new Vector(0, -100),
				size: new Vector(200, 200),
				color: color('#eeeeee00'),
			})
		)

		hacker.color(color('#eeeeee'), 0.5, Ease)

		yield* wait(0.2)

		secretKey.position(hacker.position(), 0.2, Ease)
		secretKey.size(0, 0.2, Ease)

		yield* wait(0.5)

		secretKey.position(new Vector(0, 100), 0.2, Ease)
		secretKey.size(90, 0.2, Ease)

		yield* wait(3.5)

		secretKey.color(color('#eeeeee00'), 0.5, Ease)
		hacker.color(color('#eeeeee00'), 0.5, Ease)
		SymetricEncryptionText.color(color('#eeeeee00'), 0.5, Ease)

		yield* wait(0.5)

		secretKey.position(new Vector(-500, 100))
		secretKey.size(50)
		secretKey.color(color('#eeeeee'), 0.5, Ease)

		secretKey2.position(new Vector(500, 100))
		secretKey2.size(50)
		secretKey2.color(color('#eeeeee'), 0.5, Ease)

		yield* wait(4)

		secretKey.color(color('#eeeeee00'), 0.5, Ease)
		secretKey2.color(color('#eeeeee00'), 0.5, Ease)

		yield* wait(1.5)

		computer1.position(new Vector(-700, -300), 1.5, Ease)
		computer2.position(new Vector(700, -300), 1.5, Ease)

		const leftBar = add(
			new Rect({
				position: new Vector(-400, 1080),
				size: new Vector(1, 1080),
				color: color('#eeeeee'),
			})
		)

		leftBar.position(new Vector(-400), 1, Ease)

		yield* wait(2)

		const BobText = add(
			new VectorText({
				text: 'Bob',
				position: new Vector(-700, -450),
				color: color('#eeeeee00'),
				size: 80,
			})
		)

		BobText.color(color('#eeeeee'), 1, Ease)

		yield* wait(2.3)

		aside(shake(BobText))

		yield* wait(0.7)

		const rightBar = add(
			new Rect({
				position: new Vector(400, 1080),
				size: new Vector(1, 1080),
				color: color('#eeeeee'),
			})
		)

		rightBar.position(new Vector(400), 1, Ease)

		yield* wait(1)

		const AliceText = add(
			new VectorText({
				text: 'Alice',
				position: new Vector(700, -450),
				color: color('#eeeeee00'),
				size: 80,
			})
		)
		AliceText.color(color('#eeeeee'), 1, Ease)

		yield* wait(2.8)

		aside(shake(AliceText))

		yield* wait(1.2)

		message.position(computer2.position().add(new Vector(0, 300)))
		message.color(color('#eb7734'), 0.5, Ease)

		yield* wait(1)

		yield* message.position(computer1.position().add(new Vector(0, 300)), 2, Ease)
		yield* message.color(color('#eb773400'), 0.5, Ease)
		message.position(computer2.position().add(new Vector(0, 300)))
		message.color(color('#eb7734'), 0.5, Ease)

		hacker.position(new Vector())
		hacker.color(color('#eeeeee'), 1, Ease)

		yield* message.position(new Vector(250), 2, Ease)

		yield* wait(1)

		message.color(color('#eb773400'), 1, Ease)
		hacker.color(color('#eeeeee00'), 1, Ease)

		yield* wait(2)

		const P = add(
			new VectorText({
				text: 'P',
				position: new Vector(-150, 400),
				color: color('#ebd93400'),
				size: 80,
			})
		)
		P.color(color('#ebd934'), 1, Ease)

		yield* wait(0.5)

		const G = add(
			new VectorText({
				text: 'G',
				position: new Vector(150, 400),
				color: color('#34eb6800'),
				size: 80,
			})
		)
		G.color(color('#34eb68'), 1, Ease)

		yield* wait(1)

		const PValue = add(
			new VectorText({
				text: '= 23',
				position: new Vector(-50, 395),
				color: color('#eeeeee00'),
				size: 60,
			})
		)
		PValue.color(color('#eeeeee'), 1, Ease)

		yield* wait(2)

		const GValue = add(
			new VectorText({
				text: '= 5',
				position: new Vector(230, 395),
				color: color('#eeeeee00'),
				size: 60,
			})
		)
		GValue.color(color('#eeeeee'), 1, Ease)

		yield* wait(2)

		aside(shake(BobText, 10))
		aside(shake(AliceText, -10))

		yield* wait(3)

		const b = add(
			new VectorText({
				text: 'b',
				position: new Vector(-700, -100),
				color: color('#3474eb00'),
				size: 80,
			})
		)
		b.color(color('#3474eb'), 1, Ease)

		const a = add(
			new VectorText({
				text: 'a',
				position: new Vector(700, -110),
				color: color('#eb3d3400'),
				size: 80,
			})
		)
		a.color(color('#eb3d34'), 1, Ease)

		yield* wait(5)

		const bValue = add(
			new VectorText({
				text: '= 15',
				position: new Vector(-600, -110),
				color: color('#eeeeee00'),
				size: 60,
			})
		)
		bValue.color(color('#eeeeee'), 1, Ease)

		yield* wait(1.5)

		const aValue = add(
			new VectorText({
				text: '= 6',
				position: new Vector(780, -110),
				color: color('#eeeeee00'),
				size: 60,
			})
		)
		aValue.color(color('#eeeeee'), 1, Ease)

		yield* wait(2.5)

		aside(shake(b, 10))

		yield* wait(1.3)

		aside(shake(a, 10))

		yield* wait(2.2)

		const G2 = add(
			new VectorText({
				text: 'G',
				position: new Vector(-700, 0),
				color: color('#34eb6800'),
				size: 80,
			})
		)
		G2.color(color('#34eb68'), 1, Ease)

		const G3 = add(
			new VectorText({
				text: 'G',
				position: new Vector(700, 0),
				color: color('#34eb6800'),
				size: 80,
			})
		)
		G3.color(color('#34eb68'), 1, Ease)

		yield* wait(1.5)

		const b2 = add(
			new VectorText({
				text: 'b',
				position: reactive(() => G2.position().add(new Vector(50, 30))),
				color: color('#3474eb00'),
				size: 50,
			})
		)
		b2.color(color('#3474eb'), 1, Ease)

		const a2 = add(
			new VectorText({
				text: 'a',
				position: reactive(() => G3.position().add(new Vector(50, 30))),
				color: color('#eb3d3400'),
				size: 50,
			})
		)
		a2.color(color('#eb3d34'), 1, Ease)

		yield* wait(1.5)

		const P2 = add(
			new VectorText({
				text: '% P',
				position: new Vector(-625, 0),
				color: color('#ebd93400'),
				size: 80,
			})
		)
		P2.color(color('#ebd934'), 1, Ease)
		G2.position(new Vector(-780, 0), 0.5, Ease)

		const P3 = add(
			new VectorText({
				text: '% P',
				position: new Vector(775, 0),
				color: color('#ebd93400'),
				size: 80,
			})
		)
		P3.color(color('#ebd934'), 1, Ease)
		G3.position(new Vector(620, 0), 0.5, Ease)

		yield* wait(2)

		const background2 = add(
			new Rect({
				size: new Vector(1920, 1080),
				color: color('#14141400'),
			})
		)
		background2.color(color('#141414'), 0.5, Ease)

		yield* wait(0.5)

		const ModuloNumerator = add(
			new VectorText({
				text: '5',
				position: new Vector(-80, 0),
				color: color('#eeeeee00'),
				size: 100,
			})
		)
		ModuloNumerator.color(color('#eeeeee'), 1, Ease)

		const ModuloDenominator = add(
			new VectorText({
				text: '3',
				position: new Vector(80, 0),
				color: color('#eeeeee00'),
				size: 100,
			})
		)
		ModuloDenominator.color(color('#eeeeee'), 1, Ease)

		const ModuloPercent = add(
			new VectorText({
				text: '%',
				size: 100,
				color: color('#eeeeee00'),
			})
		)
		ModuloPercent.color(color('#eeeeee'), 1, Ease)

		yield* wait(1)

		const ModuloLine = add(
			new Rect({
				size: new Vector(160, 10),
				color: color('#eeeeee00'),
			})
		)

		ModuloLine.color(color('#eeeeee'), 1, Ease)
		ModuloPercent.color(color('#eeeeee00'), 1, Ease)
		ModuloNumerator.position(new Vector(0, 80), 1, Ease)
		ModuloDenominator.position(new Vector(0, -80), 1, Ease)

		const ModuloResult = add(
			new VectorText({
				position: new Vector(250),
				text: '= 1 R',
				size: 100,
				color: color('#eeeeee00'),
			})
		)
		ModuloResult.color(color('#eeeeee'), 1, Ease)

		const ModuloRemainder = add(
			new VectorText({
				position: new Vector(405),
				text: '2',
				size: 100,
				color: color('#eeeeee00'),
			})
		)
		ModuloRemainder.color(color('#eeeeee'), 1, Ease)

		yield* wait(1)

		ModuloResult.color(color('#eeeeee00'), 1, Ease)
		ModuloNumerator.color(color('#eeeeee00'), 1, Ease)
		ModuloDenominator.color(color('#eeeeee00'), 1, Ease)
		ModuloLine.color(color('#eeeeee00'), 1, Ease)
		ModuloRemainder.position(new Vector(), 1, Ease)
		ModuloRemainder.size(150, 1, Ease)

		yield* wait(1)

		ModuloRemainder.color(color('#eeeeee00'), 0.5, Ease)
		yield* wait(0.5)
		background2.color(color('#14141400'), 0.5, Ease)

		yield* wait(4)

		const P2Offset = P2.position().subtract(G2.position())
		P2.position(reactive(() => G2.position().add(P2Offset)))
		const P3Offset = P3.position().subtract(G3.position())
		P3.position(reactive(() => G3.position().add(P3Offset)))

		G2.position(G3.position(), 1, Ease)
		G3.position(G2.position(), 1, Ease)

		yield* wait(6)

		aside(shake(b2, 15))
		aside(shake(a2, 15))

		yield* wait(14 - 6)

		G3.position(new Vector(-900, 0), 1, Ease)
		G2.position(new Vector(500, 0), 1, Ease)

		yield* wait(1)

		const BobParens = add(
			new VectorText({
				text: '(             )',
				position: new Vector(-960, 0),
				origin: new Vector(0, 0.5),
				color: color('#eeeeee00'),
				size: 80,
			})
		)
		BobParens.color(color('#eeeeee'), 1, Ease)

		const AliceParens = add(
			new VectorText({
				text: '(             )',
				position: new Vector(440, 0),
				origin: new Vector(0, 0.5),
				color: color('#eeeeee00'),
				size: 80,
			})
		)
		AliceParens.color(color('#eeeeee'), 1, Ease)

		yield* wait(0.5)

		const b3 = add(
			new VectorText({
				text: 'b',
				position: reactive(() => G3.position().add(new Vector(280, 30))),
				color: color('#3474eb00'),
				size: 50,
			})
		)
		b3.color(color('#3474eb'), 1, Ease)

		const a3 = add(
			new VectorText({
				text: 'a',
				position: reactive(() => G2.position().add(new Vector(280, 30))),
				color: color('#3474eb00'),
				size: 50,
			})
		)
		a3.color(color('#eb3d34'), 1, Ease)

		yield* wait(1.5)

		const P4 = add(
			new VectorText({
				text: '% P',
				position: new Vector(-515, 0),
				color: color('#ebd93400'),
				size: 80,
			})
		)
		P4.color(color('#ebd934'), 1, Ease)

		const P5 = add(
			new VectorText({
				text: '% P',
				position: new Vector(880, 0),
				color: color('#ebd93400'),
				size: 80,
			})
		)
		P5.color(color('#ebd934'), 1, Ease)

		yield* wait(5)

		//#3474eb
		//#ebd934
		//#34eb68
		//#eb3d34

		G3.color(color('#34eb6800'), 0.5, Ease)
		a2.color(color('#eb3d3400'), 0.5, Ease)
		b3.color(color('#3474eb00'), 0.5, Ease)
		BobParens.color(color('#eeeeee00'), 0.5, Ease)
		P3.color(color('#ebd93400'), 0.5, Ease)
		P4.color(color('#ebd93400'), 0.5, Ease)

		G2.color(color('#34eb6800'), 0.5, Ease)
		a3.color(color('#eb3d3400'), 0.5, Ease)
		b2.color(color('#3474eb00'), 0.5, Ease)
		AliceParens.color(color('#eeeeee00'), 0.5, Ease)
		P2.color(color('#ebd93400'), 0.5, Ease)
		P5.color(color('#ebd93400'), 0.5, Ease)

		yield* wait(0.5)

		const BobValue = add(
			new VectorText({
				text: '2',
				position: new Vector(-700, 0),
				color: color('#eeeeee00'),
				size: 80,
			})
		)
		BobValue.color(color('#eeeeee'), 1, Ease)

		const AliceValue = add(
			new VectorText({
				text: '2',
				position: new Vector(700, 0),
				color: color('#eeeeee00'),
				size: 80,
			})
		)
		AliceValue.color(color('#eeeeee'), 1, Ease)

		yield* wait(1)

		leftBar.color(color('#eeeeee00'), 1, Ease)
		rightBar.color(color('#eeeeee00'), 1, Ease)
		P.color(color('#ebd93400'), 1, Ease)
		G.color(color('#34eb6800'), 1, Ease)
		PValue.color(color('#eeeeee00'), 1, Ease)
		GValue.color(color('#eeeeee00'), 1, Ease)
		a.color(color('#eb3d3400'), 1, Ease)
		b.color(color('#3474eb00'), 1, Ease)
		aValue.color(color('#eeeeee00'), 1, Ease)
		bValue.color(color('#eeeeee00'), 1, Ease)

		BobValue.position(new Vector(-200), 1, Ease)
		BobValue.size(200, 1, Ease)
		AliceValue.position(new Vector(200), 1, Ease)
		AliceValue.size(200, 1, Ease)

		const ValueEquals = add(
			new VectorText({
				text: '=',
				color: color('#eeeeee00'),
				size: 200,
			})
		)
		ValueEquals.color(color('#eeeeee'), 1, Ease)

		computer1.position(new Vector(-700, 0), 1, Ease)
		computer2.position(new Vector(700, 0), 1, Ease)
		BobText.position(new Vector(-700, -150), 1, Ease)
		AliceText.position(new Vector(700, -150), 1, Ease)

		yield* wait(0.5)

		aside(shake(BobText, 3))
		aside(shake(AliceText, -3))

		yield* wait(5.5)

		message.color(color('#eb7734'), 0.5, Ease)
		message.position(new Vector(-800, 150))

		secretKey.color(color('#eeeeee'), 0.5, Ease)
		secretKey.position(new Vector(-500, 150))

		secretKey2.color(color('#eeeeee'), 0.5, Ease)
		secretKey2.position(new Vector(500, 150))

		yield* wait(0.5)

		aside(shake(BobValue, 3))
		aside(shake(AliceValue, -3))

		yield* wait(0.5)

		message.position(new Vector(800, 150), 2, Ease)
		yield* wait(0.7)
		message.text((Math.random() + 1).toString(36).substring(4))

		yield* wait(0.7)
		message.text('Oranges!')
	}
)
