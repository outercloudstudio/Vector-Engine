export async function loadAudio(path: string) {
  if (!import.meta.hot) return

  import.meta.hot.send('vector-engine:load-content', path)

  const data = await new Promise<ArrayBuffer>(res => {
    import.meta.hot.on('vector-engine:load-content', data => {
      if (data.path != path) return

      res(new Uint8Array(data.result.data).buffer)
    })
  })

  const ctx = new AudioContext()!
  const audioBuffer = await ctx.decodeAudioData(data)
  ctx.close()

  return audioBuffer
}

export async function loadImage(path: string) {
  if (!import.meta.hot) return

  import.meta.hot.send('vector-engine:load-content', path)

  const data = await new Promise<ArrayBuffer>(res => {
    import.meta.hot.on('vector-engine:load-content', data => {
      if (data.path != path) return

      res(new Uint8Array(data.result.data).buffer)
    })
  })

  const blob = new Blob([data], { type: 'image/png' })
  const url = URL.createObjectURL(blob)

  const image = await new Promise<HTMLImageElement>(res => {
    const image = new Image()

    image.addEventListener('load', () => res(image))

    image.src = url
  })

  URL.revokeObjectURL(url)

  return image
}
