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
}: any) {
  defineName('Laggy Scene')

  for (let i = 0; i < 1000000000; i++) {}
}
