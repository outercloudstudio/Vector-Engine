import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './style.css'

export default function inject(project: any, data: any) {
  createApp(App, {
    project,
    data,
  })
    .use(createPinia())
    .mount('#app')
}
