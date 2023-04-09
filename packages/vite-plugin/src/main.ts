import path from 'path'
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

  const projectFolder = path.posix.join(
    posix(path.dirname(url.fileURLToPath(configURI))),
    'src'
  )
  const project = path.posix.join(
    posix(path.dirname(url.fileURLToPath(configURI))),
    'src',
    'main.ts'
  )
  const dataFile = path.posix.join(
    posix(path.dirname(url.fileURLToPath(configURI))),
    'data.json'
  )

  return {
    name: 'vector-engine',
    resolveId(id: string) {
      console.log('💥 Resolving: ', id)

      if (id === virtualInjectPackage) {
        return resolvedVirtualInjectPackage
      } else if (id === virtualProjectPackage) {
        console.log(project)

        return project
      } else if (id === virtualDataPackage) {
        return dataFile
      }
    },
    load(id: string) {
      console.log('💾 Loading:', id)

      if (id === resolvedVirtualInjectPackage) {
        return `
        import data from 'virtual:@vector-engine/data'
        import { project } from 'virtual:@vector-engine/project'
        import { VectorEngine } from '@vector-engine/editor'

        VectorEngine()

        window.dispatchEvent(new CustomEvent('project', { detail: { project, data } }))

        import.meta.hot.on('vector-engine:project-update', project => {
          window.dispatchEvent(new CustomEvent('project-update', { detail: project }))
        })

        import.meta.hot.on('vector-engine:update-data', data => {
          window.dispatchEvent(new CustomEvent('data-update', { detail: data }))
        })

        window.addEventListener('update-data', event => {
          import.meta.hot.send('vector-engine:update-data', event.detail)
        })

        window.addEventListener('export', event => {
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
          res.setHeader('Content-Type', 'text/html')
          res.end(
            `
            <!DOCTYPE html>
            <html lang="en">
              <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />

                <link rel="icon" href="/favicon.ico" />
                <link
                  rel="stylesheet"
                  href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,300,0..1,-50..200"
                />

                <title>Vector Engine</title>
              </head>
              <body>
                <div id="app"></div>
                <script type="module" src="/@id/__x00__virtual:@vector-engine/inject"></script>
              </body>
            </html>
            `
          )

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

      if (ctx.file == dataFile) {
        ctx.server.ws.send(
          'vector-engine:update-data',
          JSON.parse(await ctx.read())
        )

        return []
      }
    },
  }
}
