import { Ease, Rect, SceneContext, Vector, waitFor } from '@vector-engine/core'

export default async function* ({
  add,
  remove,
  relative,
  frames,
  wait,
  aside,
}: SceneContext) {
  const square = add(
    new Rect({
      position: relative(new Vector(0.2, 0.5)),
      size: new Vector(500, 500),
      outline: new Vector(1, 0, 0, 1),
      outlineWidth: 10,
      radius: 50,
      rotation: 45,
    })
  )

  yield* wait(0.5)

  remove(square)

  // yield* waitForMarker('Start')

  // yield* square.animatePosition(relative(new Vector(0.8, 0.5)), 0.5, Modes.Ease)

  const rotateAside = await aside(async function* () {
    while (true) {
      yield* await square.animateRotation(-45, 1, Ease)
      yield* square.animateRotation(45, 1, Ease)
    }
  })

  yield* waitFor(rotateAside)

  yield* wait(frames(5))
}
