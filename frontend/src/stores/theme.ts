import { computed, onScopeDispose, ref, watch } from 'vue'
import { defineStore } from 'pinia'
import { useTheme } from 'vuetify'

export type ThemePreference = 'system' | 'light' | 'dark'

type MediaQueryListWithLegacyListener = MediaQueryList & {
  addListener?: (listener: (event: MediaQueryListEvent) => void) => void
  removeListener?: (listener: (event: MediaQueryListEvent) => void) => void
}

const storageKey = 'tarkov-item-manager-theme-preference'

function storedPreference(): ThemePreference {
  const value = localStorage.getItem(storageKey)
  return value === 'light' || value === 'dark' || value === 'system' ? value : 'system'
}

export const useThemeStore = defineStore('theme', () => {
  const theme = useTheme()
  const preference = ref<ThemePreference>(storedPreference())
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)') as MediaQueryListWithLegacyListener
  const resolved = computed<'light' | 'dark'>(() => preference.value === 'system'
    ? (mediaQuery.matches ? 'dark' : 'light')
    : preference.value)

  function applyTheme() {
    const dark = resolved.value === 'dark'
    theme.global.name.value = dark ? 'tarkovDark' : 'tarkovLight'
    document.documentElement.style.colorScheme = resolved.value
    document.querySelector('meta[name="theme-color"]')?.setAttribute('content', dark ? '#15191e' : '#f5f7f5')
  }

  function setPreference(value: ThemePreference) {
    preference.value = value
    localStorage.setItem(storageKey, value)
  }

  function handleSystemThemeChange() {
    if (preference.value === 'system') applyTheme()
  }

  mediaQuery.addEventListener?.('change', handleSystemThemeChange)
  mediaQuery.addListener?.(handleSystemThemeChange)
  watch(resolved, applyTheme, { immediate: true })
  onScopeDispose(() => {
    mediaQuery.removeEventListener?.('change', handleSystemThemeChange)
    mediaQuery.removeListener?.(handleSystemThemeChange)
  })

  return { preference, resolved, setPreference }
})
