import { beforeEach, describe, expect, it } from 'vitest'
import { nextTick } from 'vue'
import { useTheme } from '../useTheme'

describe('useTheme', () => {
  beforeEach(() => {
    localStorage.clear()
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
})
