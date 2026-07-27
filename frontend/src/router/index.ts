import { createRouter, createWebHistory } from 'vue-router'
import LoginView from '@/views/LoginView.vue'
import WorkspaceView from '@/views/WorkspaceView.vue'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({ history: createWebHistory(), routes: [
  { path: '/login', component: LoginView, meta: { guest: true } },
  { path: '/', component: WorkspaceView, meta: { auth: true } },
] })

router.beforeEach(async (to) => {
  const auth = useAuthStore()
  if (!auth.initialized) await auth.restore()
  if (to.meta.auth && !auth.user) return '/login'
  if (to.meta.guest && auth.user) return '/'
})

export default router
