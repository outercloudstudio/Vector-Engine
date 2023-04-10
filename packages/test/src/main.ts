import { loadAudio } from '@vector-engine/core'
import audio from '../Assets/Full.mp3'
import scene from './scene'
import laggyScene from './laggyScene'

export const project = async ({
  frameRate,
  length,
  minutes,
  seconds,
  loadScene,
  loadScenes,
  audioTrack,
}: any) => {
  frameRate(60)
  length(200)

  audioTrack(audio)

  // await loadScene(scene)

  loadScenes({
    laggyScene,
    scene,
  })
}
