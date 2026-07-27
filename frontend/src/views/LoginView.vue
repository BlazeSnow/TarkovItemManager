<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()
const registerMode = ref(false)
const username = ref('')
const password = ref('')
const loading = ref(false)
const error = ref('')
const valid = computed(() => username.value.trim().length >= 3 && password.value.length >= 8)

async function submit() {
  if (!valid.value) return
  loading.value = true; error.value = ''
  try {
    if (registerMode.value) await auth.register(username.value, password.value)
    else await auth.login(username.value, password.value)
    await router.push('/')
  } catch (reason) { error.value = reason instanceof Error ? reason.message : '操作失败' }
  finally { loading.value = false }
}
</script>

<template>
  <v-container class="auth-layout d-flex align-center justify-center">
    <v-sheet class="auth-panel pa-6" border>
      <div class="text-overline text-secondary mb-2">HIDEOUT PLANNER</div>
      <h1 class="text-h5 mb-6">{{ registerMode ? '创建本地账户' : '登录工作区' }}</h1>
      <v-alert v-if="error" class="mb-4" density="compact" type="error">{{ error }}</v-alert>
      <v-form @submit.prevent="submit">
        <v-text-field v-model="username" autocomplete="username" label="用户名" prepend-inner-icon="mdi-account" :rules="[v => v.trim().length >= 3 || '至少 3 个字符']" />
        <v-text-field v-model="password" autocomplete="current-password" label="密码" prepend-inner-icon="mdi-lock" type="password" :rules="[v => v.length >= 8 || '至少 8 个字符']" />
        <v-btn block color="primary" :disabled="!valid" :loading="loading" type="submit">{{ registerMode ? '注册并进入' : '登录' }}</v-btn>
      </v-form>
      <v-btn block class="mt-3" variant="text" @click="registerMode = !registerMode; error = ''">{{ registerMode ? '已有账户，去登录' : '没有账户，创建一个' }}</v-btn>
    </v-sheet>
  </v-container>
</template>
