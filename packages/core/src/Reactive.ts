export type OptionalReactive<T> = Reactive<T> | T

type Reactor<T> = { (): T; isReactive: boolean }

class Reactive<T> {
  static reactor: Reactive<any> | null = null
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
    if (Reactive.reactor === null) {
      Reactive.reactor = this
    } else {
      if (!this.dependants.includes(Reactive.reactor))
        this.dependants.push(Reactive.reactor)
    }

    console.log('Executing ', Reactive.reactor === this, this.valid, this.value)

    const result = this.valid ? this.value : this.expression()

    if (!this.valid) {
      this.valid = true
      this.value = result
    }

    if (Reactive.reactor === this) Reactive.reactor = null

    return result
  }

  toReactor(): Reactor<T> {
    const reactor = function (value?: T) {
      if (arguments.length !== 0) {
        console.log('Setting!', this.dependants)

        return this.set(value)
      }

      return this.execute()
    }.bind(this)

    reactor.isReactive = true

    return reactor
  }
}

export function reactive<T>(value: T | (() => T)): Reactor<T> {
  return new Reactive(value).toReactor()
}

export function ensureReactive<T>(value: Reactor<T> | T) {
  if ((<Reactor<T>>value).isReactive) return <Reactor<T>>value

  return reactive(<T>value)
}

export function unreactive<T>(value: Reactor<T> | T) {
  if ((<Reactor<T>>value).isReactive) return (<Reactor<T>>value)()

  return <T>value
}
