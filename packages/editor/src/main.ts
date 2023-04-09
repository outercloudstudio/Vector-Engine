import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './style.css'

export function VectorEngine() {
  createApp(App).use(createPinia()).mount('#app')
}
