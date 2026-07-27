import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api, type User } from '@/api'

export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(null)
  const initialized = ref(false)

  async function restore() {
    try { user.value = await api.me() } catch { user.value = null } finally { initialized.value = true }
  }
  async function login(username: string, password: string) { user.value = await api.login(username, password) }
  async function register(username: string, password: string) { user.value = await api.register(username, password) }
  async function logout() { await api.logout(); user.value = null }

  return { user, initialized, restore, login, register, logout }
})
