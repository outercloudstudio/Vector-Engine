export type Reactor<T> = { (value?: T): T; isReactive: boolean }

export type MaybeReactor<T> = Reactor<T> | T

class Reactive<T> {
	static root: Reactive<any> | null = null

	expression: () => T

	dependants: Reactive<T>[] = []

	valid: boolean = false

	value: T

	constructor(value: (() => T) | T) {
		if (typeof value === 'function') {
			this.expression = <() => T>value
		} else {
			this.expression = () => <T>value
		}

		this.value = this.execute()
	}

	set(value: T): T {
		this.value = value
		this.valid = true

		for (const dependant of this.dependants) {
			dependant.valid = false
		}

		return value
	}

	execute(): T {
		if (Reactive.root === null) {
			Reactive.root = this
		} else {
			if (!this.dependants.includes(Reactive.root)) this.dependants.push(Reactive.root)
		}

		let result = this.value

		if (!this.valid) {
			try {
				result = this.expression()
			} catch (error) {
				console.error(error)
			}
		}

		if (!this.valid) {
			this.valid = true
			this.value = result
		}

		if (Reactive.root === this) Reactive.root = null

		return result
	}

	toReactor(): Reactor<T> {
		const reactor = function (value?: T) {
			if (arguments.length !== 0) return this.set(value)

			return this.execute()
		}.bind(this)

		reactor.isReactive = true

		return reactor
	}
}

export function reactive<T>(value: T | (() => T)): Reactor<T> {
	return new Reactive(value).toReactor()
}

export function ensureReactive<T>(value: MaybeReactor<T>) {
	if ((<Reactor<T>>value).isReactive) return <Reactor<T>>value

	return reactive(<T>value)
}

export function unreactive<T>(value: MaybeReactor<T>) {
	if ((<Reactor<T>>value).isReactive) return (<Reactor<T>>value)()

	return <T>value
}
