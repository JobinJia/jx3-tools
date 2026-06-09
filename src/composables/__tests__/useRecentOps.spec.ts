import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useRecentOps } from '../useRecentOps'

describe('useRecentOps', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-03-15T14:02:00'))
    localStorage.clear()
    const { recentOps } = useRecentOps()
    recentOps.value = []
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('adds newest record first', () => {
    const { recentOps, addRecentOp } = useRecentOps()
    addRecentOp('角色甲', '角色乙')
    addRecentOp('角色丙', '角色丁')
    expect(recentOps.value).toHaveLength(2)
    expect(recentOps.value[0]!.source).toBe('角色丙')
    expect(recentOps.value[1]!.target).toBe('角色乙')
  })

  it('caps records at 10', () => {
    const { recentOps, addRecentOp } = useRecentOps()
    for (let i = 0; i < 13; i++)
      addRecentOp(`s${i}`, `t${i}`)
    expect(recentOps.value).toHaveLength(10)
    expect(recentOps.value[0]!.source).toBe('s12')
  })

  it('formats today as HH:mm, yesterday as 昨天, older as M/D', () => {
    const { formatOpTime } = useRecentOps()
    expect(formatOpTime(new Date('2026-03-15T14:02:00').getTime())).toBe('14:02')
    expect(formatOpTime(new Date('2026-03-15T08:05:00').getTime())).toBe('08:05')
    expect(formatOpTime(new Date('2026-03-14T22:00:00').getTime())).toBe('昨天')
    expect(formatOpTime(new Date('2026-03-12T10:00:00').getTime())).toBe('3/12')
  })

  it('handles month boundary for yesterday', () => {
    vi.setSystemTime(new Date('2026-03-01T09:00:00'))
    const { formatOpTime } = useRecentOps()
    expect(formatOpTime(new Date('2026-02-28T23:59:00').getTime())).toBe('昨天')
    expect(formatOpTime(new Date('2026-02-27T12:00:00').getTime())).toBe('2/27')
  })
})
