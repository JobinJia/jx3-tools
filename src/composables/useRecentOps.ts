import { useStorage } from '@vueuse/core'

export interface RecentOp {
  source: string
  target: string
  at: number
}

const MAX_RECENT_OPS = 10

// 模块级单例（与 useKeyboard 的共享模式一致）
const recentOps = useStorage<RecentOp[]>('keyboard-recent-ops', [])

export function useRecentOps() {
  function addRecentOp(source: string, target: string) {
    recentOps.value.unshift({ source, target, at: Date.now() })
    if (recentOps.value.length > MAX_RECENT_OPS)
      recentOps.value.length = MAX_RECENT_OPS
  }

  function formatOpTime(at: number): string {
    const d = new Date(at)
    const today = new Date()
    if (d.toDateString() === today.toDateString()) {
      const hh = String(d.getHours()).padStart(2, '0')
      const mm = String(d.getMinutes()).padStart(2, '0')
      return `${hh}:${mm}`
    }
    const yesterday = new Date(today)
    yesterday.setDate(today.getDate() - 1)
    if (d.toDateString() === yesterday.toDateString())
      return '昨天'
    return `${d.getMonth() + 1}/${d.getDate()}`
  }

  return { recentOps, addRecentOp, formatOpTime }
}
