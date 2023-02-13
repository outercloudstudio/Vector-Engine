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

  console.log(
    editorFolder,
    editorDistFolder,
    editorIndexPath,
    import.meta.url,
    import.meta,
    configURI,
    projectFolder,
    project
  )

  return {
    name: 'vector-engine',
    resolveId(id: string) {
      console.log('Resolving:', id)

      if (id === virtualProjectPackage) {
        return resolvedVirtualProjectPackage
      }
    },
    load(id: string) {
      console.log('Loading:', id)

      if (id === resolvedVirtualProjectPackage)
        return `
        import inject from '@vector-engine/editor'
        import { project } from '${project}'
    
        inject(project)
        `
    },
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        console.log('Fetching:', req.url)

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
          !req.url.startsWith('/@') &&
          !req.url.startsWith('/node_modules/')
        ) {
          if (fs.existsSync(path.join(editorDistFolder, req.url))) {
            console.warn('Fetching from fs:', req.url)

            if (req.url.endsWith('.js'))
              res.setHeader('Content-Type', 'text/javascript')

            res.end(fs.readFileSync(path.join(editorDistFolder, req.url)))
            return
          } else {
            console.warn('Tried to fetch file that does not exist!', req.url)
          }
        }

        next()
      })
    },
  }
}
