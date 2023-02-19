import Perlin from './perlinTest'

export default async ({
  frameRate,
  length,
  minutes,
  seconds,
  loadScene,
}: any) => {
  frameRate(60)
  length(seconds(1))
  await loadScene(Perlin)
  // audioTrack('Assets/test.wav')
}
