import audio from '../Assets/Full.mp3'
import scene from './scene'

export const project = async ({
  frameRate,
  length,
  minutes,
  seconds,
  loadScene,
  audioTrack,
}: any) => {
  frameRate(60)
  length(seconds(3))

  audioTrack(audio)

  await loadScene(scene)
}
