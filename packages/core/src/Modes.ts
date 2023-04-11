export function Linear(time: number): number {
  return time
}

export function Ease(time: number): number {
  return time < 0.5 ? 2 * time * time : 1 - Math.pow(-2 * time + 2, 2) / 2
}

export function EaseIn(time: number): number {
  return Math.pow(time, 3)
}

export function EaseOut(time: number): number {
  return 1 - Math.pow(1 - time, 3)
}
