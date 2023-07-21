import { Scene } from '../assets/scene'
import { Interpolator as InterpolationFunction, TimingFunction, interpolate } from '../interpolate'
import { MaybeReactor, Reactor, ensureReactive, reactive, unreactive } from '../reactive'

export class Element {
	public scene: Scene | null = null

	public async render(canvas: OffscreenCanvas) {}
}

export type AnimatedReactiveProperty<PropertyType> = {
	(): PropertyType
	(value: MaybeReactor<PropertyType>): void
	(value: MaybeReactor<PropertyType>, length: number, timingFunction: TimingFunction): Generator
}

export function animatedReactiveProperty<PropertyType>(
	defaultValue: PropertyType,
	interpolationFunction: InterpolationFunction<PropertyType> = interpolate
): AnimatedReactiveProperty<PropertyType> {
	let internalPropertyValue = reactive<PropertyType>(defaultValue)

	return <AnimatedReactiveProperty<PropertyType>>function (value, length, timingFunction) {
		if (value === undefined) return internalPropertyValue()

		if (length === undefined) {
			if ((<Reactor<PropertyType>>value).isReactive) {
				internalPropertyValue = <Reactor<PropertyType>>value
			} else {
				internalPropertyValue(<PropertyType>value)
			}
			return
		}

		const from = internalPropertyValue()
		const to = unreactive(value)

		const context = (function* () {
			for (let frame = 1; frame <= Math.ceil(length * 60) - 1; frame++) {
				internalPropertyValue(
					interpolationFunction(from, to, timingFunction(frame / Math.ceil(length * 60)))
				)

				yield null
			}

			internalPropertyValue(to)
		})()

		;(<Element>this).scene.aside(context)

		return context
	}
}
