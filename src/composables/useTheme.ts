import type { GlobalTheme } from 'naive-ui'
import { useStorage } from '@vueuse/core'
import { darkTheme } from 'naive-ui'
import { computed, ref, watchEffect } from 'vue'

export type ThemeMode = 'system' | 'light' | 'dark'

// 模块级单例状态（与 useKeyboard 的共享模式一致）
const mode = useStorage<ThemeMode>('jx3-theme-mode', 'system')

// 系统主题：jsdom 没有 matchMedia，需判空（不要用 naive 的 useOsTheme，它依赖组件生命周期）
const prefersDark = typeof window.matchMedia === 'function'
  ? window.matchMedia('(prefers-color-scheme: dark)')
  : null
const osDark = ref(prefersDark?.matches ?? false)
prefersDark?.addEventListener('change', (e) => {
  osDark.value = e.matches
})

const isDark = computed(() =>
  mode.value === 'system' ? osDark.value : mode.value === 'dark',
)

// 根元素 .dark class 同步（模块加载时注册一次；detached effect，随应用整个生命周期存活，故不保留 stop handle）
watchEffect(() => {
  document.documentElement.classList.toggle('dark', isDark.value)
})

const naiveTheme = computed<GlobalTheme | null>(() => (isDark.value ? darkTheme : null))

export function useTheme() {
  function cycleMode() {
    mode.value = mode.value === 'system' ? 'light' : mode.value === 'light' ? 'dark' : 'system'
  }

  return { mode, isDark, naiveTheme, cycleMode }
}
