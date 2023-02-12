import { createApp } from 'vue'
import App from './App.vue'
import './style.css'

export default function inject(test: string) {
  createApp(App, {
    project: test,
  }).mount('#app')
}
