import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from './api'

afterEach(() => {
  vi.unstubAllGlobals()
})

function jsonResponse(status: number, body: unknown) {
  return new Response(JSON.stringify(body), { status, headers: { 'Content-Type': 'application/json' } })
}

describe('api request wrapper', () => {
  it('returns parsed JSON on success', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => jsonResponse(200, { id: 1, username: 'alice' })))
    await expect(api.me()).resolves.toEqual({ id: 1, username: 'alice' })
  })

  it('throws the server error message on failure', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => jsonResponse(401, { error: '请先登录' })))
    await expect(api.me()).rejects.toThrow('请先登录')
  })

  it('falls back to a generic message when the body is not JSON', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('gateway timeout', { status: 502 })))
    await expect(api.me()).rejects.toThrow('请求失败')
  })

  it('sends cookies and returns undefined for 204 responses', async () => {
    const fetchMock = vi.fn(async () => new Response(null, { status: 204 }))
    vi.stubGlobal('fetch', fetchMock)
    await expect(api.logout()).resolves.toBeUndefined()
    expect(fetchMock).toHaveBeenCalledWith('/api/auth/logout', expect.objectContaining({ credentials: 'include', method: 'POST' }))
  })

  it('sends JSON bodies for saving progress', async () => {
    const fetchMock = vi.fn(async () => new Response(null, { status: 204 }))
    vi.stubGlobal('fetch', fetchMock)
    await api.saveFacilityLevels([{ facilityId: 3, level: 2 }])
    expect(fetchMock).toHaveBeenCalledWith('/api/progress/facilities', expect.objectContaining({
      method: 'PUT',
      body: JSON.stringify([{ facilityId: 3, level: 2 }]),
    }))
  })
})
