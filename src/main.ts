import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { initRuntimes } from 'bridge-js-runtime'
import wasmUrl from '@swc/wasm-web/wasm-web_bg.wasm?url'
import './style.css'
import App from '@/App.vue'
import router from '@/router'

initRuntimes(wasmUrl)

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.mount('#app')
