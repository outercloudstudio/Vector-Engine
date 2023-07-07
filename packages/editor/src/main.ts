import './app.css'
import App from './App.svelte'

const app = new App({
	target: document.getElementById('app'),
})

document.addEventListener('@vector-engine/project-reload', (event: CustomEvent) => {
	console.log(event.detail)
})

export default app
