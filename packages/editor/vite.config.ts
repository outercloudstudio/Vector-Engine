import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

export default defineConfig({
  // build: {
  //   lib: {
  //     entry: ['index.html', 'src/inject.ts'],
  //     formats: ['es'],
  //     fileName: (format: any, entryName: string) => {
  //       if (entryName == 'index') return 'index.js'

  //       if (entryName == 'inject') return 'inject.js'
  //     },
  //   },
  // },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  plugins: [vue()],
})
