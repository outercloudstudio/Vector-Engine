import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import VectorEnginePlugin from '@vector-engine/vite-plugin'

// https://vitejs.dev/config/
export default defineConfig({
	plugins: [svelte(), VectorEnginePlugin()],
})
