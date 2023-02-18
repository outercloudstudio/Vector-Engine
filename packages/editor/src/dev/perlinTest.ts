import * as noise from './perlin'
export default async function* ({
  Builders,
  createElement,
  Vector,
  relative,
  ElementTransitions,
  removeElement,
  importImage,
  Modes,
  waitForMarker,
  wait,
  waitForAll,
  animate,
  Element,
  frames,
  transition,
  Transitions,
  defineName,
  aside,
}: any) {
  defineName('Perlin Noise')

  // for (let x = 0; x < 16; x++) {
  //   for (let y = 0; y < 16; y++) {
  //     // createElement(Builders.Ellipse, {
  //     //   position: new Vector(x * 60 + 30, y * 60 + 30),
  //     //   size: new Vector(60, 60),
  //     //   color: new Vector(0, 0, 0, 1),
  //     // })

  //     yield wait(1)
  //     yield wait(1)
  //     yield wait(1)
  //     yield wait(1)
  //     yield wait(1)
  //   }
  // }

  createElement(Builders.Rect, {
    origin: new Vector(),
    size: relative(new Vector(1, 1)),
    color: new Vector(20 / 255, 20 / 255, 20 / 255, 1),
    priority: -1,
  })

  const stageLabel = createElement(Builders.Text, {
    text: 'Biomes',
    position: relative(new Vector(0, 1)).add(new Vector(75, -50)),
    origin: new Vector(0, 0.5),
    size: 30,
    color: new Vector(217, 217, 217, 1).divide(new Vector(255, 255, 255, 1)),
  })

  const perlinNoiseLabel = createElement(Builders.Text, {
    text: 'Perlin Noise',
    position: relative(new Vector(0.5, 0.76)),
    size: 0,
    color: new Vector(217, 217, 217, 1).divide(new Vector(255, 255, 255, 1)),
  })

  yield perlinNoiseLabel.animateSize(30, 0.5, Modes.EaseOut)

  const scale = 2
  const amount = 16
  const length = 1227 - 1098
  const speed = 1
  let noiseCache: number[][][] = []
  let noisePoints: typeof Element[][] = []

  for (let x = -amount / 2; x <= amount / 2; x++) {
    for (let y = -amount / 2; y <= amount / 2; y++) {
      for (let frame = 0; frame < length; frame++) {
        if (noiseCache[frame] == undefined) {
          noiseCache[frame] = []
        }

        if (noiseCache[frame][x + amount / 2] == undefined) {
          noiseCache[frame][x + amount / 2] = []
        }

        if (noisePoints[x + amount / 2] == undefined) {
          noisePoints[x + amount / 2] = []
        }

        noiseCache[frame][x + amount / 2][y + amount / 2] =
          ((<any>noise.noise).perlin3(
            ((x + amount / 2) / amount) * scale,
            ((y + amount / 2) / amount) * scale,
            (frame / length) * speed
          ) +
            0.5) /
          2
      }

      noisePoints[x + amount / 2][y + amount / 2] = createElement(
        Builders.Ellipse,
        {
          position: relative(new Vector(0.5, 0.5)).add(
            new Vector(x * 30, y * 30)
          ),
          size: new Vector(),
          color: new Vector(0, 0, 0, 1),
        }
      )

      aside(
        (async function* () {
          yield* wait(
            0.02 * (x + amount / 2 + amount * 2 - (y + amount / 2)) - 0.2
          )

          yield* noisePoints[x + amount / 2][y + amount / 2].animateSize(
            new Vector(30, 30),
            0.4,
            Modes.EaseOut
          )
        })()
      )
    }
  }

  for (let frame = 0; frame < length; frame++) {
    for (let x = -amount / 2; x <= amount / 2; x++) {
      for (let y = -amount / 2; y <= amount / 2; y++) {
        const noise = noiseCache[frame][x + amount / 2][y + amount / 2]

        noisePoints[x + amount / 2][y + amount / 2].position = relative(
          new Vector(0.5, 0.5)
        ).add(new Vector(x * 30, y * 30))

        noisePoints[x + amount / 2][y + amount / 2].scale = new Vector(
          noise / 2 + 0.5,
          noise / 2 + 0.5
        )

        noisePoints[x + amount / 2][y + amount / 2].color = new Vector(
          noise,
          noise,
          noise,
          1
        )
      }
    }

    yield null
  }

  for (let x = -amount / 2; x <= amount / 2; x++) {
    for (let y = -amount / 2; y <= amount / 2; y++) {
      aside(
        (async function* () {
          yield* wait(
            0.02 * (x + amount / 2 + amount * 2 - (y + amount / 2)) - 0.2
          )

          yield* noisePoints[x + amount / 2][y + amount / 2].animateSize(
            new Vector(),
            0.4,
            Modes.EaseIn
          )

          removeElement(noisePoints[x + amount / 2][y + amount / 2])
        })()
      )
    }
  }

  yield* perlinNoiseLabel.animateSize(0, 0.5, Modes.EaseIn)

  yield* waitForMarker('Vorinoi Noise')

  removeElement(perlinNoiseLabel)

  // yield transition('Vorinoi', Transitions.Cut)
}
