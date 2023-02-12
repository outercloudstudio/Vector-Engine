import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import fs from 'fs'

// https://vitejs.dev/config/
export default defineConfig({
  build: {
    lib: {
      entry: 'src/inject.ts',
      formats: ['es'],
      fileName: 'main',
    },
  },
  plugins: [
    // @ts-ignore
    vue(),
    {
      name: 'copy-files',
      async buildStart() {
        this.emitFile({
          type: 'asset',
          fileName: 'index.html',
          source: await fs.promises.readFile('./inject.html'),
        })
      },
    },
  ],
})
