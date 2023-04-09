import { VectorEngine } from '@vector-engine/editor'
import scene from './src/scene'

const project = VectorEngine(
  async ({ frameRate, length, loadScene, audioTrack }: any) => {
    frameRate(60)
    length(2)

    await loadScene(scene)
  }
)

// import audio from '../Assets/Full.mp3'

// export const project = async ({
//   frameRate,
//   length,
//   minutes,
//   seconds,
//   loadScene,
//   audioTrack,
// }: any) => {
//   frameRate(60)
//   length(2)

//   audioTrack(audio)

//   await loadScene(scene)
// }
