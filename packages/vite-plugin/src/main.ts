import path from 'path'
import { resolve } from 'import-meta-resolve'
import fs from 'fs'
import url from 'url'

function posix(pathStr: string) {
  return pathStr.split(path.sep).join(path.posix.sep)
}

export default async function VectorEngine(configURI: string) {
  const virtualInjectPackage = 'virtual:@vector-engine/inject'
  const resolvedVirtualInjectPackage = '\0' + virtualInjectPackage

  const virtualProjectPackage = 'virtual:@vector-engine/project'

  const virtualDataPackage = 'virtual:@vector-engine/data'

  const editorDistFolder = path.posix.join(
    posix(
      path.dirname(
        url.fileURLToPath(
          await resolve('@vector-engine/editor', import.meta.url)
        )
      )
    ),
    'dist'
  )

  const editorIndexPath = path.posix.join(editorDistFolder, 'index.html')

  const projectFolder = posix(path.dirname(url.fileURLToPath(configURI)))
  const project = path.posix.join(projectFolder, '/src/main.ts')
  const dataFile = path.posix.join(projectFolder, '/data.json')

  let indexFile = ''

  return {
    name: 'vector-engine',
    resolveId(id: string) {
      console.log('💥 Resolving:', id)

      if (id === virtualInjectPackage) {
        return resolvedVirtualInjectPackage
      } else if (id === virtualProjectPackage) {
        return project
      } else if (id === virtualDataPackage) {
        return dataFile
      } else if (id.startsWith('/')) {
        return path.posix.join(editorDistFolder, id)
      }
    },
    load(id: string) {
      console.log('💾 Loading:', id)

      if (id === resolvedVirtualInjectPackage) {
        return `
        import * as index from '${indexFile}'
        import data from 'virtual:@vector-engine/data'
        import { project } from 'virtual:@vector-engine/project'

        window.dispatchEvent(new CustomEvent('project', { detail: { project, data } }))

        import.meta.hot.on('vector-engine:update-data', data => {
          window.dispatchEvent(new CustomEvent('on:data-update', { detail: data }))
        })
        
        window.addEventListener('send:update-data', event => {
          import.meta.hot.send('vector-engine:update-data', event.detail)
        })

        window.addEventListener('send:export', event => {
          import.meta.hot.send('vector-engine:export', event.detail)
        })
        `
      }
    },
    transform(code, id) {
      console.log('⚙️ Transforming: ', id)

      if (id.endsWith('.mp3') || id.endsWith('.wav')) {
        return `
        import { loadAudio } from '@vector-engine/core'
        export default await loadAudio('${id}')
        `
      } else if (id.endsWith('.png')) {
        return `
        import { loadImage } from '@vector-engine/core'
        export default await loadImage('${id}')
        `
      } else if (id == project) {
        return (
          code +
          `
        import.meta.hot.accept(newModule => {
          window.dispatchEvent(new CustomEvent('project-update', { detail: newModule.project }))
        })
        `
        )
      }
    },
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const [base, query] = req.url.split('?')
        const params = new URLSearchParams(query)

        console.log('❓ Fetching:', req.url)

        if (req.url === '/') {
          let html = fs.readFileSync(editorIndexPath).toString()

          const relativeIndexFile = html
            .split('<script type="module" crossorigin src="')[1]
            .split('"></script>')[0]

          html = html.replace(
            relativeIndexFile,
            '/@id/__x00__virtual:@vector-engine/inject'
          )

          indexFile = path.posix.join(editorDistFolder, relativeIndexFile)

          res.setHeader('Content-Type', 'text/html')
          res.end(html)

          return
        } else if (req.url.startsWith('/assets')) {
          res.end(fs.readFileSync(path.posix.join(editorDistFolder, req.url)))

          return
        }

        next()
      })

      server.ws.on('vector-engine:update-data', (data, client) => {
        fs.writeFileSync(dataFile, JSON.stringify(data, null, 2))
      })

      server.ws.on('vector-engine:load-content', (data, client) => {
        client.send('vector-engine:load-content', {
          path: data,
          result: fs.readFileSync(data),
        })
      })

      server.ws.on('vector-engine:export', (data, client) => {
        const { name, image } = data

        if (!fs.existsSync(path.posix.join(projectFolder, 'Exports')))
          fs.mkdirSync(path.posix.join(projectFolder, 'Exports'))

        if (
          !fs.existsSync(
            path.posix.join(projectFolder, 'Exports', path.posix.dirname(name))
          )
        )
          fs.mkdirSync(
            path.posix.join(projectFolder, 'Exports', path.posix.dirname(name))
          )

        try {
          const imageArray = []
          for (const item of Object.values(image)) {
            imageArray.push(item)
          }

          const buffer = Buffer.from(imageArray)

          fs.writeFileSync(
            path.posix.join(projectFolder, 'Exports', name),
            buffer
          )
        } catch (err) {
          console.log(err)
        }
      })
    },
    async handleHotUpdate(ctx) {
      console.log('⚠️ HMR update for ', ctx.file)
      console.log(ctx.modules)

      if (ctx.file == dataFile) {
        // console.log('HMR updating data file!')

        ctx.server.ws.send(
          'vector-engine:update-data',
          JSON.parse(await ctx.read())
        )

        return []
      }
      // else if (ctx.file.startsWith(projectFolder)) {
      //   // console.log('HMR updating project file!')

      //   ctx.server.ws.send('vector-engine:update-project')

      //   return []
      // }
    },
  }
}
