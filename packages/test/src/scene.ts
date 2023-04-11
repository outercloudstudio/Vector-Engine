import testImage from '../Assets/image.png'

import {
  Ease,
  Ellipse,
  Rect,
  SceneContext,
  Vector,
  VectorImage,
  VectorText,
  color,
} from '@vector-engine/core'

export default async function* ({ add, relative, aside }: SceneContext) {
  const dot = add(
    new Ellipse({
      position: relative(new Vector(0.2, 0.5)),
      size: new Vector(10, 10),
    })
  )

  const text = add(
    new VectorText({
      position: relative(new Vector(0.2, 0.5)),
      rotation: 45,
    })
  )

  const rotateAside = await aside(async function* () {
    while (true) {
      yield* await text.animateRotation(-45, 1, Ease)
      yield* text.animateRotation(45, 1, Ease)
    }
  })

  const image = add(
    new VectorImage({
      image: testImage,
      position: relative(new Vector(0.8, 0.5)),
      color: color('#00FF00'),
      // rotation: 45,
    })
  )

  // yield* square.animatePosition(relative(new Vector(0.8, 0.5)), 1, Ease)
  // yield* square.animatePosition(relative(new Vector(0.2, 0.5)), 1, Ease)
}
