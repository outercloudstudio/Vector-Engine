import { defineConfig } from 'vite'
import VectorEngine from '@vector-engine/vite-plugin'
import { svelte } from '@sveltejs/vite-plugin-svelte'

export default defineConfig({
	optimizeDeps: {
		include: ['@vector-engine/vite-plugin'],
	},
	plugins: [VectorEngine(import.meta.url), svelte()],
	server: {
		hmr: true,
		fs: {
			strict: false,
		},
	},
})
