export const project = async ({
  frameRate,
  length,
  minutes,
  seconds,
  loadScene,
}: any) => {
  frameRate(60)
  length(seconds(1))
  await loadScene(async function* ({
    Vector,
    Builders,
    createElement,
    Modes,
    Transitions,
    transition,
    relative,
    waitForMarker,
    defineName,
  }: any) {
    defineName('Sample Scene')

    const square = createElement(Builders.Rect, {
      position: relative(new Vector(0.2, 0.5)),
      size: new Vector(500, 500),
      outlineColor: new Vector(1, 0, 0, 1),
      outlineWidth: 10,
      radius: 50,
    })

    yield* waitForMarker('Start')

    yield* square.animatePosition(
      relative(new Vector(0.8, 0.5)),
      0.5,
      Modes.Ease
    )

    yield* transition(async function* ({
      Vector,
      Builders,
      createElement,
      Modes,
      relative,
      waitForMarker,
      defineName,
    }: any) {
      defineName('Sample Scene 2')

      const square = createElement(Builders.Rect, {
        position: relative(new Vector(0.2, 0.5)),
        size: new Vector(500, 500),
        outlineColor: new Vector(0, 1, 0, 1),
        outlineWidth: 10,
        radius: 50,
      })

      yield* square.animatePosition(
        relative(new Vector(0.8, 0.5)),
        0.5,
        Modes.Ease
      )
    },
    Transitions.Cut)
  })
  // audioTrack('Assets/test.wav')
}
