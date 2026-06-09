import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import { useTheme } from '../useTheme'

describe('useTheme', () => {
  beforeEach(() => {
    localStorage.clear()
    const { mode } = useTheme()
    mode.value = 'system'
    document.documentElement.classList.remove('dark')
  })

  it('cycles mode system -> light -> dark -> system', () => {
    const { mode, cycleMode } = useTheme()
    mode.value = 'system'
    cycleMode()
    expect(mode.value).toBe('light')
    cycleMode()
    expect(mode.value).toBe('dark')
    cycleMode()
    expect(mode.value).toBe('system')
  })

  it('toggles .dark class on documentElement', async () => {
    const { mode } = useTheme()
    mode.value = 'dark'
    await nextTick()
    expect(document.documentElement.classList.contains('dark')).toBe(true)
    mode.value = 'light'
    await nextTick()
    expect(document.documentElement.classList.contains('dark')).toBe(false)
  })

  it('exposes naive dark theme only when dark', async () => {
    const { mode, naiveTheme, isDark } = useTheme()
    mode.value = 'light'
    await nextTick()
    expect(isDark.value).toBe(false)
    expect(naiveTheme.value).toBeNull()
    mode.value = 'dark'
    await nextTick()
    expect(isDark.value).toBe(true)
    expect(naiveTheme.value).not.toBeNull()
  })

  it('follows OS dark preference in system mode', async () => {
    vi.resetModules()
    const listeners: Array<(e: { matches: boolean }) => void> = []
    vi.stubGlobal('matchMedia', (query: string) => ({
      matches: true,
      media: query,
      addEventListener: (_type: string, cb: (e: { matches: boolean }) => void) => {
        listeners.push(cb)
      },
    }))
    try {
      const fresh = await import('../useTheme')
      const { mode, isDark } = fresh.useTheme()
      mode.value = 'system'
      await nextTick()
      expect(isDark.value).toBe(true)

      listeners.forEach(cb => cb({ matches: false }))
      await nextTick()
      expect(isDark.value).toBe(false)
    } finally {
      vi.unstubAllGlobals()
    }
  })
})
