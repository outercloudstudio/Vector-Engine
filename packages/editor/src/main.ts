import inject from './inject.js'
import './style.css'

inject(async ({ frameRate, length, minutes, seconds, loadScene }: any) => {
  frameRate(60)
  length(seconds(1))
  await loadScene(async function* ({
    Vector,
    Builders,
    createElement,
    Modes,
    Transitions,
    transition,
    ElementTransitions,
    removeElement,
    relative,
  }: any) {
    const square = createElement(Builders.Rect, {
      position: relative(new Vector(0.2, 0.5)),
      size: new Vector(500, 500),
      outlineColor: new Vector(1, 0, 0, 1),
      outlineWidth: 10,
      radius: 50,
    })

    yield* square.animatePosition(relative(new Vector(0.8, 0.5)), 1, Modes.Ease)
  })
  // audioTrack('Assets/test.wav')
})
