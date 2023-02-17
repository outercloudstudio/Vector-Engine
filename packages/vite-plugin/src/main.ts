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

  console.log(import.meta.url, configURI)

  const editorFolder = posix(
    path.join(
      path.dirname(
        url.fileURLToPath(
          await resolve('@vector-engine/editor', import.meta.url)
        )
      ),
      '..'
    )
  )
  const editorDistFolder = path.posix.join(editorFolder, 'dist')
  const editorIndexPath = path.posix.join(editorDistFolder, 'index.html')

  const projectFolder = posix(path.dirname(url.fileURLToPath(configURI)))
  const project = path.posix.join(projectFolder, '/src/main.ts')
  const dataFile = path.posix.join(projectFolder, '/data.json')

  console.log(projectFolder, project, dataFile)

  return {
    name: 'vector-engine',
    resolveId(id: string) {
      console.log('Resolving:', id)

      if (id === virtualInjectPackage) {
        return resolvedVirtualInjectPackage
      } else if (id === virtualProjectPackage) {
        return project
      } else if (id === virtualDataPackage) {
        return dataFile
      } else if (id.startsWith('@/')) {
        return path.posix.join('/@fs/', editorFolder, '/src', id.substring(2))
      }
    },
    load(id: string) {
      console.log('Loading:', id)

      if (id === resolvedVirtualInjectPackage) {
        return `
        import inject from '@vector-engine/editor'
        import data from 'virtual:@vector-engine/data'
        import { project } from 'virtual:@vector-engine/project'
    
        const app = inject(project, data)

        import.meta.hot.on('vector-engine:update-data', data => {
          window.dispatchEvent(new CustomEvent('on:data-update', { detail: data }))
        })

        window.addEventListener('send:update-data', event => {
          import.meta.hot.send('vector-engine:update-data', event.detail)
        })
        `
      }
    },
    transform(code, id) {
      console.log('Transforming: ', id)

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
        console.log('Fetching:', req.url)

        const [base, query] = req.url.split('?')
        const params = new URLSearchParams(query)

        if (req.url === '/') {
          res.setHeader('Content-Type', 'text/html')

          res.end(
            fs
              .readFileSync(editorIndexPath)
              .toString()
              .replace(
                '{{source}}',
                '/@id/__x00__virtual:@vector-engine/inject'
              )
          )
          return
        } else if (
          (!req.url.startsWith('/@') || req.url.startsWith('/@/')) &&
          !req.url.startsWith('/node_modules/')
        ) {
          const url = req.url.startsWith('/@/')
            ? path.posix.join(editorFolder, '/src', req.url.substring(3))
            : path.posix.join(editorDistFolder, req.url)
          if (fs.existsSync(url)) {
            console.warn('    Fetching from fs:', url)

            if (url.endsWith('.js'))
              res.setHeader('Content-Type', 'text/javascript')

            res.end(fs.readFileSync(url))
            return
          } else {
            console.warn('    Tried to fetch file that does not exist!', url)
          }
        }

        next()
      })

      server.ws.on('vector-engine:update-data', (data, client) => {
        console.log('Writing updated data: ', dataFile, data)

        fs.writeFileSync(dataFile, JSON.stringify(data, null, 2))
      })

      server.ws.on('vector-engine:load-content', (data, client) => {
        client.send('vector-engine:load-content', {
          path: data,
          result: fs.readFileSync(data),
        })
      })
    },
    async handleHotUpdate(ctx) {
      console.log('HRM update for ', ctx.file, ctx.modules)

      if (ctx.file == dataFile) {
        console.log('HMR updating data file!')

        ctx.server.ws.send(
          'vector-engine:update-data',
          JSON.parse(await ctx.read())
        )

        return []
      } else if (ctx.file.startsWith(projectFolder)) {
        console.log('HMR updating project file!')

        ctx.server.ws.send('vector-engine:update-project')

        // return []
      }
    },
  }
}
