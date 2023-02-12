import path from 'path';
import { resolve } from 'import-meta-resolve';
import fs from 'fs';
import url from 'url';
export default async function VectorEngine() {
    const editorFolder = path.join(path.dirname(url.fileURLToPath(await resolve('@vector-engine/editor', import.meta.url))), '..');
    const editorDistFolder = path.join(editorFolder, 'dist');
    const editorIndexPath = path.join(editorDistFolder, 'index.html');
    console.log(editorFolder, editorDistFolder, editorIndexPath);
    return {
        name: 'vector-engine',
        load(id) {
            if (id === '\0virtual:@vector-engine/test')
                return `
        import { test } from '@vector-engine/editor'
        alert(test)
        `;
        },
        configureServer(server) {
            server.middlewares.use((req, res, next) => {
                console.log(req.url);
                if (req.url === '/') {
                    res.setHeader('Content-Type', 'text/html');
                    res.end(fs
                        .readFileSync(editorIndexPath)
                        .toString()
                        .replace('</body>', `<script type="module" src="/@id/__x00__virtual:@vector-engine/test"></script></body>`));
                    return;
                }
                else if (!req.url.startsWith('/@') &&
                    !req.url.startsWith('/node_modules/')) {
                    if (req.url.endsWith('.js'))
                        res.setHeader('Content-Type', 'text/javascript');
                    res.end(fs.readFileSync(path.join(editorDistFolder, req.url)));
                    return;
                }
                next();
            });
        },
    };
}
