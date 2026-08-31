<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { api, type AppVersion } from '@/api'

const fallbackRepository = 'https://github.com/BlazeSnow/TarkovItemManager'
const version = ref<AppVersion | null>(null)

const currentPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const saving = ref(false)
const message = ref('')
const messageType = ref<'success' | 'error'>('success')
const valid = computed(() => currentPassword.value.length >= 8 && newPassword.value.length >= 8 && newPassword.value === confirmPassword.value)

async function submit() {
  if (!valid.value || saving.value) return
  saving.value = true
  try {
    await api.changePassword(currentPassword.value, newPassword.value)
    currentPassword.value = ''
    newPassword.value = ''
    confirmPassword.value = ''
    messageType.value = 'success'
    message.value = '密码已更新，其他设备的登录会话已退出'
  } catch (reason) {
    messageType.value = 'error'
    message.value = reason instanceof Error ? reason.message : '密码修改失败'
  } finally {
    saving.value = false
  }
}

onMounted(async () => {
  try {
    version.value = await api.version()
  } catch {
    version.value = null
  }
})
</script>

<template>
  <v-container class="py-6">
    <div class="mb-6"><div class="text-overline text-secondary">SETTINGS</div><h1 class="text-h4">设置</h1></div>
    <v-row>
      <v-col cols="12" md="6">
        <v-sheet class="pa-6" color="surface" border>
          <div class="text-overline text-secondary mb-1">ACCOUNT</div>
          <h2 class="text-h6 mb-4">修改密码</h2>
          <v-alert v-if="message" class="mb-4" closable density="compact" :type="messageType" @click:close="message = ''">{{ message }}</v-alert>
          <v-form @submit.prevent="submit">
            <v-text-field v-model="currentPassword" autocomplete="current-password" label="当前密码" prepend-inner-icon="mdi-lock-outline" type="password" :rules="[v => v.length >= 8 || '至少 8 个字符']" />
            <v-text-field v-model="newPassword" autocomplete="new-password" label="新密码" prepend-inner-icon="mdi-key-variant" type="password" :rules="[v => v.length >= 8 || '至少 8 个字符']" />
            <v-text-field v-model="confirmPassword" autocomplete="new-password" label="确认新密码" prepend-inner-icon="mdi-lock-check-outline" type="password" :rules="[v => v === newPassword || '两次输入的密码不一致']" />
            <v-btn block class="mt-2" color="primary" :disabled="!valid" :loading="saving" type="submit">更新密码</v-btn>
          </v-form>
        </v-sheet>
      </v-col>
      <v-col cols="12" md="6">
        <v-sheet class="pa-6" color="surface" border>
          <div class="text-overline text-secondary mb-1">ABOUT</div>
          <h2 class="text-h6 mb-4">关于</h2>
          <div class="about-row"><span class="text-medium-emphasis">软件名称</span><span class="font-weight-medium">{{ version?.name ?? 'Tarkov Item Manager' }}</span></div>
          <div class="about-row"><span class="text-medium-emphasis">软件版本</span><span class="font-weight-medium">{{ version?.version ?? 'dev' }}</span></div>
          <div class="about-row"><span class="text-medium-emphasis">软件仓库</span><a :href="version?.repository ?? fallbackRepository" target="_blank" rel="noreferrer">{{ version?.repository ?? fallbackRepository }}</a></div>
        </v-sheet>
      </v-col>
    </v-row>
  </v-container>
</template>
