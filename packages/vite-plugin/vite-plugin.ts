import type { Plugin } from 'vite'

import { posix, resolve, sep, parse, join, basename } from 'path'
import {
	createReadStream,
	existsSync,
	readFileSync,
	readdirSync,
	statSync,
	writeFileSync,
} from 'fs'
import { watch } from 'chokidar'
import { createHash } from 'crypto'
import { Meta } from '@vector-engine/core'

function toPosix(path: string) {
	return path.split(sep).join(posix.sep)
}

export default function VectorEngine(): Plugin {
	const metaFilePath = resolve('./vector-engine.meta.json')

	function makeMetaFile() {
		const meta: Meta = {
			assets: {},
		}

		const foldersToRead: string[] = []

		foldersToRead.push(resolve('./assets/'))

		while (foldersToRead.length > 0) {
			const currentFolder = foldersToRead.shift()

			for (const item of readdirSync(currentFolder)) {
				const itemPath = join(currentFolder, item)

				if (itemPath === resolve('./assets/vector-engine.d.ts')) continue

				if (statSync(itemPath).isFile()) {
					meta.assets[createHash('sha256').update(itemPath).digest('hex')] = {
						path: itemPath,
						name: basename(itemPath),
					}
				} else {
					foldersToRead.push(itemPath)
				}
			}
		}

		writeFileSync(metaFilePath, JSON.stringify(meta, null, 2))
	}

	function getMetaFile(): Meta {
		if (!existsSync(metaFilePath)) makeMetaFile()

		return JSON.parse(readFileSync(metaFilePath).toString())
	}

	return {
		name: 'vector-engine',
		resolveId(id, importer, options) {
			if (id === 'virtual:@vector-engine/editor') {
				return '\0virtual:@vector-engine/editor'
			}

			if (id === 'virtual:@vector-engine/assets') {
				return '\0virtual:@vector-engine/assets'
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

			if (id === '\0virtual:@vector-engine/assets') {
				makeMetaFile()
				const meta = getMetaFile()

				let assetImports = ''
				let assetObject = '{'

				for (const assetId of Object.keys(meta.assets)) {
					const asset = meta.assets[assetId]

					assetImports += `import asset_${assetId} from '${toPosix(asset.path)}'\n`

					assetObject += `\n\t"${assetId}": asset_${assetId},`
				}

				assetObject += '\n}'

				return `
        ${assetImports}

        export default ${assetObject}
        `
			}

			if (id === '\0virtual:@vector-engine/inject') {
				return `
        import assets from 'virtual:@vector-engine/assets'
        import * as meta from '${posix.resolve('./vector-engine.meta.json')}'

        import.meta.hot.accept()

        document.dispatchEvent(new CustomEvent('@vector-engine/project-reload', { detail: { assets, meta } } ))
        `
			}
		},
		transform(code, id) {
			if (id.endsWith('.png') || id.endsWith('.jpg')) {
				return `
        import { image } from '@vector-engine/core'

        export default image('${id}')
        `
			}

			if (id.endsWith('.mp4')) {
				return `
        import { video } from '@vector-engine/core'

        export default video('${id}')
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
                <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200" />
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

				if (base === '/@asset') {
					const type = params.get('type')
					const path = params.get('path')

					if (type === 'image') {
						res.setHeader('Content-Type', 'image/png')
					}

					if (type === 'video') {
						if (path.endsWith('.mp4')) res.setHeader('Content-Type', 'video/mp4')
					}

					res.end(readFileSync(path))

					return
				}

				next()
			})

			watch(resolve('./assets/'), {})
				.on('add', (event, path) => {
					const module = server.moduleGraph.getModuleById('\0virtual:@vector-engine/assets')

					if (module === undefined) return

					makeMetaFile()

					server.reloadModule(module)
				})
				.on('unlink', (event, path) => {
					const module = server.moduleGraph.getModuleById('\0virtual:@vector-engine/assets')

					if (module === undefined) return

					makeMetaFile()

					server.reloadModule(module)
				})

			server.ws.on('@vector-engine/image', (data, client) => {
				client.send('@vector-engine/image', readFileSync(data.path))
			})

			server.ws.on('@vector-engine/video', (data, client) => {
				client.send('@vector-engine/video', readFileSync(data.path))
			})

			server.ws.on('@vector-engine/export', (data, client) => {
				let { frame, dataUrl } = data

				const base64Data = dataUrl.slice(dataUrl.indexOf(',') + 1)
				writeFileSync(
					join(posix.resolve('./export/'), 'frame_' + frame.toString().padStart(4, '0') + '.png'),
					base64Data,
					{ encoding: 'base64' }
				)
			})
		},
	}
}
