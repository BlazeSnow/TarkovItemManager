<script setup lang="ts">
import { useAuthStore } from '@/stores/auth'
const auth = useAuthStore()
const sourceUrl = import.meta.env.VITE_SOURCE_URL || 'https://github.com/'
</script>

<template>
  <v-app>
    <v-app-bar color="surface" density="comfortable" flat border>
      <v-app-bar-title class="font-weight-bold">藏身处物品管理器</v-app-bar-title>
      <template v-if="auth.user" #append>
        <span class="text-body-2 mr-3">{{ auth.user.username }}</span>
        <v-btn aria-label="退出登录" icon="mdi-logout" variant="text" @click="auth.logout().then(() => $router.push('/login'))" />
      </template>
    </v-app-bar>
    <v-main><router-view /></v-main>
    <v-footer class="text-caption d-flex justify-space-between" color="surface" border>
      <span>Escape from Tarkov Hideout Planner</span>
      <a :href="sourceUrl" target="_blank" rel="noreferrer">源代码</a>
    </v-footer>
  </v-app>
</template>
