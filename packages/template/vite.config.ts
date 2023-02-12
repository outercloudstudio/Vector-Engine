import { defineConfig } from 'vite'
import VectorEngine from '@vector-engine/vite-plugin'
import vue from '@vitejs/plugin-vue'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [VectorEngine(), vue()],
})
