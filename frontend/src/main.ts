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
    defaultTheme: 'tarkovDark',
    themes: {
      tarkovDark: {
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
      tarkovLight: {
        dark: false,
        colors: {
          background: '#f5f7f5',
          surface: '#ffffff',
          primary: '#277a52',
          secondary: '#9b6a10',
          error: '#bf3e2d',
          success: '#277a52',
        },
      },
    },
  },
})

createApp(App).use(createPinia()).use(router).use(vuetify).mount('#app')
