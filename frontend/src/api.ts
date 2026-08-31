export interface User { id: number; username: string }
export interface Requirement { itemId: number; name: string; quantity: number; foundInRaid: boolean }
export interface FacilityGate { facilityId: number; name: string; level: number; satisfied: boolean }
export interface MerchantGate { merchantId: number; name: string; level: number; satisfied: boolean }
export interface SkillGate { name: string; level: number; satisfied: boolean }
export interface Upgrade { level: number; available: boolean; constructionTimeSeconds: number; requirements: Requirement[]; facilityPrerequisites: FacilityGate[]; merchantPrerequisites: MerchantGate[]; skillPrerequisites: SkillGate[]; sourceRequirementsAvailable: boolean }
export interface Facility { id: number; name: string; maxLevel: number; currentLevel: number; upgrades: Upgrade[] }
export interface Material { itemId: number; name: string; quantity: number; foundInRaid: boolean }
export interface LevelEntry { id: number; name: string; level: number }
export interface SkillEntry { name: string; maxLevel: number; level: number }
export interface Catalog { schemaVersion: number; gameMode: string; retrievedAt: string; facilities: Facility[]; materials: Material[]; merchants: LevelEntry[]; skills: SkillEntry[] }
export interface AppVersion { name: string; version: string; repository: string }

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(path, { credentials: 'include', headers: { 'Content-Type': 'application/json', ...options.headers }, ...options })
  if (!response.ok) { const body = await response.json().catch(() => ({ error: '请求失败' })) as { error?: string }; throw new Error(body.error ?? '请求失败') }
  if (response.status === 204) return undefined as T
  return response.json() as Promise<T>
}

export const api = {
  register: (username: string, password: string) => request<User>('/api/auth/register', { method: 'POST', body: JSON.stringify({ username, password }) }),
  login: (username: string, password: string) => request<User>('/api/auth/login', { method: 'POST', body: JSON.stringify({ username, password }) }),
  logout: () => request<void>('/api/auth/logout', { method: 'POST' }), me: () => request<User>('/api/auth/me'),
  changePassword: (currentPassword: string, newPassword: string) => request<void>('/api/auth/password', { method: 'PUT', body: JSON.stringify({ currentPassword, newPassword }) }),
  version: () => request<AppVersion>('/api/version'),
  catalog: () => request<Catalog>('/api/catalog'),
  saveFacilityLevels: (values: { facilityId: number; level: number }[]) => request<void>('/api/progress/facilities', { method: 'PUT', body: JSON.stringify(values) }),
  saveMerchantLevels: (values: { merchantId: number; level: number }[]) => request<void>('/api/progress/merchants', { method: 'PUT', body: JSON.stringify(values) }),
  saveSkillLevels: (values: { name: string; level: number }[]) => request<void>('/api/progress/skills', { method: 'PUT', body: JSON.stringify(values) }),
}
