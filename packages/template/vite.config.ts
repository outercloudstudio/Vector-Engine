import { defineConfig } from 'vite'
import VectorEngine from '@vector-engine/vite-plugin'
import vue from '@vitejs/plugin-vue'

// https://vitejs.dev/config/
export default defineConfig({
  optimizeDeps: {
    include: ['@vector-engine/vite-plugin'],
  },
  plugins: [VectorEngine(import.meta.url), vue()],
})
