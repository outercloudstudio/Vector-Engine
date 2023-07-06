import type { Plugin } from 'vite'

import { posix, resolve, sep } from 'path'

function toPosix(path: string) {
	return path.split(sep).join(posix.sep)
}

export default function VectorEngine(): Plugin {
	return {
		name: 'vector-engine',
		resolveId(id, importer, options) {
			console.log('💥 Resolving: ', id, importer)
			if (id === 'virtual:@vector-engine/inject') {
				return '\0virtual:@vector-engine/inject'
			}
		},
		load(id) {
			console.log('💾 Loading:', id)

			console.log(posix.resolve('./main.ts'))

			if (id === '\0virtual:@vector-engine/inject') {
				return `
        import editor from '@vector-engine/editor'
        import project from '${posix.resolve('./project.ts')}'
        `
			}
		},
		transform(code, id) {
			console.log('⚙️ Transforming: ', id)

			if (id.startsWith(toPosix(resolve('./'))) && id.endsWith('.ts')) {
				return (
					code +
					`
			      import.meta.hot.accept()
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
		},
	}
}
