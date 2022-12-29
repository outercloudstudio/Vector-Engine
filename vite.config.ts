import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  resolve: {
    alias: {
      '@': 'D:/Vector Engine/src',
    },
  },
  plugins: [vue()],
})
