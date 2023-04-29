import { SceneContext } from '@vector-engine/core'

export default async function* ({
  Vector,
  Builders,
  createElement,
  Modes,
  Transitions,
  transition,
  relative,
  waitForMarker,
  defineName,
}: SceneContext) {
  defineName('Laggy Scene')

  for (let i = 0; i < 1000000000; i++) {}
}
