import { ProjectContext } from '@vector-engine/core'
import audio from '../Assets/Full.mp3'
import scene from './scene'
import laggyScene from './laggyScene'

export const project = async ({
  frameRate,
  length,
  loadScenes,
  audioTrack,
}: ProjectContext) => {
  frameRate(60)
  length(120)

  audioTrack(audio)

  loadScenes({
    laggyScene,
    scene,
  })
}
