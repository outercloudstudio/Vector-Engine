import path from 'path';
import fs from 'fs';
import url from 'url';
import { spawn } from 'child_process';
import ffmpeg from 'ffmpeg-static';
function posix(pathStr) {
    return pathStr.split(path.sep).join(path.posix.sep);
}
export default async function VectorEngine(configURI) {
    const virtualInjectPackage = 'virtual:@vector-engine/inject';
    const resolvedVirtualInjectPackage = '\0' + virtualInjectPackage;
    const virtualProjectPackage = 'virtual:@vector-engine/project';
    const virtualDataPackage = 'virtual:@vector-engine/data';
    const projectBase = posix(path.dirname(url.fileURLToPath(configURI)));
    const project = path.posix.join(posix(path.dirname(url.fileURLToPath(configURI))), 'src', 'main.ts');
    const dataFile = path.posix.join(posix(path.dirname(url.fileURLToPath(configURI))), 'data.json');
    return {
        name: 'vector-engine',
        resolveId(id, importer, options) {
            console.log('💥 Resolving: ', id, importer);
            if (id === virtualInjectPackage) {
                return resolvedVirtualInjectPackage;
            }
            else if (id === virtualProjectPackage) {
                console.log(project);
                return project;
            }
            else if (id === virtualDataPackage) {
                return dataFile;
            }
        },
        load(id) {
            console.log('💾 Loading:', id);
            if (id === resolvedVirtualInjectPackage) {
                return `
        import data from 'virtual:@vector-engine/data'
        import project from 'virtual:@vector-engine/project'
        import editor from '@vector-engine/editor-new'

        window.dispatchEvent(new CustomEvent('project', { detail: { project, data } }))

        import.meta.hot.on('vector-engine:update-data', data => {
          window.dispatchEvent(new CustomEvent('data-update', { detail: data }))
        })

        window.addEventListener('update-data', event => {
          import.meta.hot.send('vector-engine:update-data', event.detail)
        })

        window.addEventListener('export-start', event => {
          import.meta.hot.send('vector-engine:export-start', event.detail)
        })

        window.addEventListener('export', event => {
          import.meta.hot.send('vector-engine:export', event.detail)
        })

        window.addEventListener('export-complete', event => {
          import.meta.hot.send('vector-engine:export-complete', event.detail)
        })
        `;
            }
        },
        transform(code, id) {
            console.log('⚙️ Transforming: ', id);
            if (id.endsWith('.mp3') || id.endsWith('.wav')) {
                return `
        import { loadAudio } from '@vector-engine/core'
        export default await loadAudio('${id}')
        `;
            }
            else if (id.endsWith('.png')) {
                return `
        import { loadImage } from '@vector-engine/core'
        export default await loadImage('${id}')
        `;
            }
            else if (id.endsWith('.mp4')) {
                console.log('Video');
                console.log(id);
                return `
        import { loadVideo } from '@vector-engine/core'
        export default await loadVideo('${id}')
        `;
            }
            else if (id == project) {
                return (code +
                    `
          import.meta.hot.accept(newModule => {
            window.dispatchEvent(new CustomEvent('project-update', { detail: newModule.default }))
          })
          `);
            }
        },
        configureServer(server) {
            server.middlewares.use((req, res, next) => {
                const [base, query] = req.url.split('?');
                const params = new URLSearchParams(query);
                console.log('❓ Fetching:', req.url);
                if (req.url === '/') {
                    res.setHeader('Content-Type', 'text/html');
                    res.end(`
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
            `);
                    return;
                }
                next();
            });
            server.ws.on('vector-engine:update-data', (data, client) => {
                fs.writeFileSync(dataFile, JSON.stringify(data, null, 2));
            });
            server.ws.on('vector-engine:load-content', (data, client) => {
                console.log('🛣️ Loading Content: ', data);
                try {
                    client.send('vector-engine:load-content', {
                        path: data,
                        result: fs.readFileSync(data),
                    });
                }
                catch (error) {
                    console.error(error);
                }
            });
            server.ws.on('vector-engine:export-start', (data, client) => {
                const { name } = data;
                const exportsFolder = path.posix.join(projectBase, 'Exports');
                const exportFolder = path.posix.join(exportsFolder, name);
                if (!fs.existsSync(exportsFolder))
                    fs.mkdirSync(exportsFolder);
                if (fs.existsSync(exportFolder))
                    fs.rmSync(exportFolder, {
                        recursive: true,
                    });
                fs.mkdirSync(exportFolder);
            });
            server.ws.on('vector-engine:export', (data, client) => {
                const { name, image } = data;
                const exportsFolder = path.posix.join(projectBase, 'Exports');
                try {
                    const imageArray = [];
                    for (const item of Object.values(image)) {
                        imageArray.push(item);
                    }
                    const buffer = Buffer.from(imageArray);
                    fs.writeFileSync(path.posix.join(exportsFolder, name), buffer);
                }
                catch (err) {
                    console.log(err);
                }
            });
            server.ws.on('vector-engine:export-complete', async (data, client) => {
                const { name, length, frameRate } = data;
                const exportsFolder = path.posix.join(projectBase, 'Exports');
                const exportFolder = path.posix.join(exportsFolder, name);
                spawn(ffmpeg, [
                    '-r',
                    frameRate,
                    '-s',
                    '1920x1080',
                    '-i',
                    path.join(exportFolder, `frame_%0${length.toString().length}d.png`),
                    '-vcodec',
                    'libx264',
                    '-crf',
                    '25',
                    '-pix_fmt',
                    'yuv420p',
                    path.join(exportFolder, `${name}.mp4`),
                ]);
            });
        },
        async handleHotUpdate(ctx) {
            if (ctx.file.startsWith(path.posix.join(projectBase, 'Exports')))
                return;
            console.log('⚠️ HMR update for ', ctx.file);
            if (ctx.file == dataFile) {
                ctx.server.ws.send('vector-engine:update-data', JSON.parse(await ctx.read()));
                return [];
            }
        },
    };
}
//# sourceMappingURL=main.js.map