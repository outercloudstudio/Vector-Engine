import { listen } from '@tauri-apps/api/event'
import './styles.css'
import App from './App.svelte'

const app = new App({
	target: document.getElementById('app'),
})

await listen('test', event => {
	console.log(event.payload)
})

export default app
