<script setup lang="ts">
import { computed } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useThemeStore, type ThemePreference } from '@/stores/theme'

const auth = useAuthStore()
const theme = useThemeStore()
const sourceUrl = 'https://github.com/BlazeSnow/TarkovItemManager'
const themeIcon = computed(() => theme.resolved === 'dark' ? 'mdi-weather-night' : 'mdi-white-balance-sunny')
const themeOptions: { value: ThemePreference; label: string; icon: string }[] = [
  { value: 'system', label: '跟随系统', icon: 'mdi-theme-light-dark' },
  { value: 'light', label: '浅色', icon: 'mdi-white-balance-sunny' },
  { value: 'dark', label: '深色', icon: 'mdi-weather-night' },
]
</script>

<template>
  <v-app>
    <v-app-bar color="surface" density="comfortable" flat border>
      <v-app-bar-title class="font-weight-bold">Tarkov Item Manager</v-app-bar-title>
      <template #append>
        <v-menu location="bottom end">
          <template #activator="{ props }">
            <v-btn v-bind="props" :aria-label="`切换主题，当前为${theme.resolved === 'dark' ? '深色' : '浅色'}`" :icon="themeIcon" variant="text" />
          </template>
          <v-list density="compact" min-width="150">
            <v-list-item v-for="option in themeOptions" :key="option.value" :active="theme.preference === option.value" @click="theme.setPreference(option.value)">
              <template #prepend><v-icon :icon="option.icon" /></template>
              <v-list-item-title>{{ option.label }}</v-list-item-title>
            </v-list-item>
          </v-list>
        </v-menu>
        <template v-if="auth.user">
          <span class="text-body-2 mr-3">{{ auth.user.username }}</span>
          <v-btn aria-label="退出登录" icon="mdi-logout" variant="text" @click="auth.logout().then(() => $router.push('/login'))" />
        </template>
      </template>
    </v-app-bar>
    <v-main><router-view /></v-main>
    <v-footer class="text-caption d-flex justify-space-between" color="surface" border>
      <span>Tarkov Item Manager</span>
      <a :href="sourceUrl" target="_blank" rel="noreferrer">源代码</a>
    </v-footer>
  </v-app>
</template>
