import path from 'path'
import { resolve } from 'import-meta-resolve'
import fs from 'fs'
import url from 'url'

function posix(pathStr: string) {
  return pathStr.split(path.sep).join(path.posix.sep)
}

export default async function VectorEngine(configURI: string) {
  const virtualProjectPackage = 'virtual:@vector-engine/project'
  const resolvedVirtualProjectPackage = '\0' + virtualProjectPackage

  const virtualDataPackage = 'virtual:@vector-engine/data'

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

  return {
    name: 'vector-engine',
    resolveId(id: string) {
      // console.log('Resolving:', id)

      if (id === virtualProjectPackage) {
        return resolvedVirtualProjectPackage
      } else if (id === virtualDataPackage) {
        return path.posix.join(projectFolder, '/data.json')
      } else if (id.startsWith('@/')) {
        return path.posix.join(editorFolder, '/src', id.substring(2))
      }
    },
    load(id: string) {
      // console.log('Loading:', id)

      if (id === resolvedVirtualProjectPackage) {
        return `
        import inject from '@vector-engine/editor'
        import data from 'virtual:@vector-engine/data'
        import { project } from '${project}'
    
        inject(project, data)
        `
      }
    },
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        // console.log('Fetching:', req.url)

        if (req.url === '/') {
          res.setHeader('Content-Type', 'text/html')

          res.end(
            fs
              .readFileSync(editorIndexPath)
              .toString()
              .replace(
                '{{source}}',
                '/@id/__x00__virtual:@vector-engine/project'
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

          // console.log('Fetching from fs:', url)

          if (fs.existsSync(url)) {
            console.warn('Fetching from fs:', url)

            if (url.endsWith('.js'))
              res.setHeader('Content-Type', 'text/javascript')

            res.end(fs.readFileSync(url))
            return
          } else {
            console.warn('Tried to fetch file that does not exist!', url)
          }
        }

        next()
      })

      server.ws.on('vector-engine:update_data', (data, client) => {
        fs.writeFileSync(dataFile, JSON.stringify(data, null, 2))
      })
    },
    async handleHotUpdate(ctx) {
      console.log('HRM update for ', ctx.file, ctx.modules)

      if (ctx.file == dataFile) {
        console.log('HMR updating data file!')

        ctx.server.ws.send(
          'vector-engine:update_data',
          JSON.parse(await ctx.read())
        )

        return []
      }
    },
  }
}
