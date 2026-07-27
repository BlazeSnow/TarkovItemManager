import 'vuetify/styles'
import '@mdi/font/css/materialdesignicons.css'
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createVuetify } from 'vuetify'
import App from './App.vue'
import router from './router'
import './styles.css'

const vuetify = createVuetify({
  theme: {
    defaultTheme: 'tarkov',
    themes: {
      tarkov: {
        dark: true,
        colors: {
          background: '#15191e',
          surface: '#20262d',
          primary: '#74c69d',
          secondary: '#d4a64b',
          error: '#e76f51',
          success: '#74c69d',
        },
      },
    },
  },
})

createApp(App).use(createPinia()).use(router).use(vuetify).mount('#app')
