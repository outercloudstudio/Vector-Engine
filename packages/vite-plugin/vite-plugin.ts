import type { Plugin } from 'vite'

import { posix, resolve, sep, parse } from 'path'
import { readdirSync } from 'fs'
import { watch } from 'chokidar'

function toPosix(path: string) {
	return path.split(sep).join(posix.sep)
}

export default function VectorEngine(): Plugin {
	return {
		name: 'vector-engine',
		resolveId(id, importer, options) {
			if (id === 'virtual:@vector-engine/editor') {
				return '\0virtual:@vector-engine/editor'
			}

			if (id === 'virtual:@vector-engine/scenes') {
				return '\0virtual:@vector-engine/scenes'
			}

			if (id === 'virtual:@vector-engine/inject') {
				return '\0virtual:@vector-engine/inject'
			}
		},
		load(id) {
			if (id === '\0virtual:@vector-engine/editor') {
				return `
        import editor from '@vector-engine/editor'
        `
			}

			if (id === '\0virtual:@vector-engine/scenes') {
				const sceneDirectory = resolve('./src/scenes/')

				let sceneImports = ''
				let sceneObject = '{'

				for (const scene of readdirSync(sceneDirectory)) {
					const name = parse(scene).name

					if (!/^[A-Za-z0-9_]+$/.test(name)) continue

					sceneImports += `import ${name} from '${posix.join(
						posix.resolve('./src/scenes/'),
						toPosix(scene)
					)}'\n`

					sceneObject += `\n\t${name},`
				}

				sceneObject += '\n}'

				return `
        ${sceneImports}

        export default ${sceneObject}
        `
			}

			if (id === '\0virtual:@vector-engine/inject') {
				return `
        import * as project from '${posix.resolve('./src/project.ts')}'
        import scenes from 'virtual:@vector-engine/scenes'

        import.meta.hot.accept()

        document.dispatchEvent(new CustomEvent('@vector-engine/project-reload', { detail: { project: project, scenes: scenes } } ))
        `
			}
		},
		configureServer(server) {
			server.middlewares.use((req, res, next) => {
				const [base, query] = req.url.split('?')
				const params = new URLSearchParams(query)

				if (req.url === '/') {
					res.setHeader('Content-Type', 'text/html')

					res.end(
						`
            <!DOCTYPE html>
            <html lang="en">
              <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <link rel="icon" href="data:,">
                <link
                  rel="stylesheet"
                  href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,300,0..1,-50..200"
                />
                <title>Vector Engine</title>
              </head>
              <body>
                <div id="app"></div>
                <script type="module" src="/@id/__x00__virtual:@vector-engine/editor"></script>
                <script type="module" src="/@id/__x00__virtual:@vector-engine/inject"></script>
              </body>
            </html>
            `
					)

					return
				}

				next()
			})

			watch(resolve('./src/scenes/'))
				.on('add', (event, path) => {
					const module = server.moduleGraph.getModuleById('\0virtual:@vector-engine/scenes')

					if (module === undefined) return

					server.reloadModule(module)
				})
				.on('unlink', (event, path) => {
					const module = server.moduleGraph.getModuleById('\0virtual:@vector-engine/scenes')

					if (module === undefined) return

					server.reloadModule(module)
				})
		},
	}
}
