export interface User { id: number; username: string }
export interface Prerequisite { facilityId: string; facilityName: string; level: number; satisfied: boolean }
export interface Facility { id: string; name: string; max_level: number; selected_level: number; prerequisites: Prerequisite[] }
export interface Material { id: string; name: string; quantity: number; checked: boolean }
export interface Catalog { facilities: Facility[]; materials: Material[] }

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(path, { credentials: 'include', headers: { 'Content-Type': 'application/json', ...options.headers }, ...options })
  if (!response.ok) {
    const body = await response.json().catch(() => ({ error: '请求失败' })) as { error?: string }
    throw new Error(body.error ?? '请求失败')
  }
  if (response.status === 204) return undefined as T
  return response.json() as Promise<T>
}

export const api = {
  register: (username: string, password: string) => request<User>('/api/auth/register', { method: 'POST', body: JSON.stringify({ username, password }) }),
  login: (username: string, password: string) => request<User>('/api/auth/login', { method: 'POST', body: JSON.stringify({ username, password }) }),
  logout: () => request<void>('/api/auth/logout', { method: 'POST' }),
  me: () => request<User>('/api/auth/me'),
  catalog: () => request<Catalog>('/api/catalog'),
  saveFacilities: (values: { facilityId: string; level: number }[]) => request<void>('/api/progress/facilities', { method: 'PUT', body: JSON.stringify(values) }),
  saveMaterials: (itemIds: string[]) => request<void>('/api/progress/materials', { method: 'PUT', body: JSON.stringify({ itemIds }) }),
}
