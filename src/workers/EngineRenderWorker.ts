import { initRuntimes } from 'bridge-js-runtime'
import wasmUrl from '@swc/wasm-web/wasm-web_bg.wasm?url'
import { Engine } from '@/engine/Engine'
import { Runtime } from '@/Runtime'

initRuntimes(wasmUrl)

let engine: Engine | null = null
let runtime: Runtime | null = null

let renderCache: OffscreenCanvas[] = []

async function frame() {
  if (!engine) return
  if (!runtime) return

  if (engine.frame == engine.length) return

  await engine.next()
  const render = await engine.render()

  renderCache[engine.frame] = render

  requestAnimationFrame(frame)

  // console.log('Cached frame ' + engine.frame)
}

onmessage = async message => {
  if (message.data.event == 'load') {
    console.log('Caching Engine!')

    renderCache = []

    runtime = new Runtime(null)
    runtime.reload(message.data.projectFolder)

    engine = new Engine(
      runtime,
      message.data.markers,
      false,
      (error: string) => {
        console.warn('Error from render worker:')
        console.error(error)
      }
    )

    await engine.load()

    console.log('Loaded Engine!', engine.length)

    const render = await engine.render()
    renderCache[0] = render

    console.log('Cached frame ' + engine.frame)

    frame()
  } else if (message.data.event == 'render') {
    async function update() {
      if (renderCache[message.data.frame]) {
        const render = await renderCache[
          message.data.frame
        ].transferToImageBitmap()

        console.log('Returning frame ' + message.data.frame)
        postMessage(render, <any>[render])

        return
      }

      requestAnimationFrame(update)
    }

    update()
  }
}

export {}
