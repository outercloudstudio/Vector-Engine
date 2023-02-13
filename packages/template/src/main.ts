export const project = ({
  frameRate,
  length,
  minutes,
  seconds,
  scenes,
  initialScene,
  audioTrack,
}: any) => {
  frameRate(60)
  length(seconds(8))
  scenes({
    Links: 'Src/Links.ts',
    Scene1: 'Src/Scene1.ts',
    Scene2: 'Src/Scene2.ts',
  })
  initialScene('Links')
  // audioTrack('Assets/test.wav')
}
