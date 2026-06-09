import { beforeEach, describe, expect, it } from 'vitest'
import { useRecentOps } from '../useRecentOps'

describe('useRecentOps', () => {
  beforeEach(() => {
    localStorage.clear()
    const { recentOps } = useRecentOps()
    recentOps.value = []
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

  it('formats today as HH:mm and older days as M/D', () => {
    const { formatOpTime } = useRecentOps()
    const now = new Date()
    now.setHours(14, 2, 0, 0)
    expect(formatOpTime(now.getTime())).toBe('14:02')

    const old = new Date(now)
    old.setDate(old.getDate() - 3)
    expect(formatOpTime(old.getTime())).toBe(`${old.getMonth() + 1}/${old.getDate()}`)
  })
})
