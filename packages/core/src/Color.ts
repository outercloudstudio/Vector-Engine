import { Vector } from './Vector'

export function color(hex: string): Vector {
  const r = parseInt(hex.substring(1, 3), 16) / 255
  const g = parseInt(hex.substring(3, 5), 16) / 255
  const b = parseInt(hex.substring(5, 7), 16) / 255
  const a = hex.length == 9 ? parseInt(hex.substring(7, 9), 16) / 255 : 1

  return new Vector(r, g, b, a)
}
