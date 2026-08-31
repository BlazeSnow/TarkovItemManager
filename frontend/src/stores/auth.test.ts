import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { api } from '@/api'
import { useAuthStore } from './auth'

vi.mock('@/api', () => ({
  api: { me: vi.fn(), login: vi.fn(), register: vi.fn(), logout: vi.fn() },
}))

const mockedApi = vi.mocked(api, true)

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
})

describe('auth store', () => {
  it('restores the user when the session is valid', async () => {
    mockedApi.me.mockResolvedValue({ id: 1, username: 'alice' })
    const store = useAuthStore()
    await store.restore()
    expect(store.user?.username).toBe('alice')
    expect(store.initialized).toBe(true)
  })

  it('clears the user when the session is invalid', async () => {
    mockedApi.me.mockRejectedValue(new Error('请先登录'))
    const store = useAuthStore()
    await store.restore()
    expect(store.user).toBeNull()
    expect(store.initialized).toBe(true)
  })

  it('stores the user after login and clears it after logout', async () => {
    mockedApi.login.mockResolvedValue({ id: 2, username: 'bob' })
    mockedApi.logout.mockResolvedValue(undefined)
    const store = useAuthStore()
    await store.login('bob', 'password123')
    expect(store.user?.username).toBe('bob')
    await store.logout()
    expect(store.user).toBeNull()
    expect(mockedApi.logout).toHaveBeenCalledOnce()
  })
})
