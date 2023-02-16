export async function loadAudio(path: string) {
  if (!import.meta.hot) return

  import.meta.hot.send('vector-engine:load-content', path)

  const data = await new Promise<ArrayBuffer>(res => {
    import.meta.hot.on('vector-engine:load-content', data => {
      res(new Uint8Array(data.data).buffer)
    })
  })

  const ctx = new AudioContext()!
  const audioBuffer = await ctx.decodeAudioData(data)
  ctx.close()

  return audioBuffer
}
