onmessage = message => {
  let volumePerFrame: number[] = []

  const samplesPerFrame = message.data.sampleRate / message.data.frameRate

  let peak = 0

  for (let frame = 0; frame < message.data.length; frame++) {
    const startingSample = Math.floor(samplesPerFrame * frame)
    let frameValue = 0

    for (const channel of message.data.channels) {
      for (
        let sample = startingSample;
        sample <
        Math.min(startingSample + Math.floor(samplesPerFrame), channel.length);
        sample++
      ) {
        frameValue = Math.max(Math.abs(channel[sample]), frameValue)
      }
    }

    const squaredValue = Math.pow(frameValue, 3)

    volumePerFrame.push(squaredValue)

    peak = Math.max(squaredValue, peak)
  }

  for (let frame = 0; frame < message.data.length; frame++) {
    volumePerFrame[frame] = Math.min(
      volumePerFrame[frame] / Math.max(peak, 0.0001),
      1
    )
  }

  postMessage(volumePerFrame)

  self.close()
}

export {}
