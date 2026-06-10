import { describe, expect, it } from 'vitest'
import { summarizePluginSync } from '../useKeyboard'

describe('summarizePluginSync', () => {
  it('merges synced dirs into one success message', () => {
    const result = summarizePluginSync({
      synced: ['my#data', 'SG#data'],
      skipped: [],
    })
    expect(result.success).toBe('插件配置已同步: my#data、SG#data')
    expect(result.warnings).toEqual([])
  })

  it('turns each skipped dir into a warning with its reason', () => {
    const result = summarizePluginSync({
      synced: ['my#data'],
      skipped: [
        { dir: 'lm#data', reason: '目标角色在该插件下无数据（请先用目标角色登录一次游戏）' },
        { dir: 'SG#data', reason: '源角色在该插件下无数据' },
      ],
    })
    expect(result.success).toBe('插件配置已同步: my#data')
    expect(result.warnings).toEqual([
      'lm#data: 目标角色在该插件下无数据（请先用目标角色登录一次游戏）',
      'SG#data: 源角色在该插件下无数据',
    ])
  })

  it('warns when nothing was synced at all', () => {
    const result = summarizePluginSync({ synced: [], skipped: [] })
    expect(result.success).toBeUndefined()
    expect(result.warnings).toEqual(['未发现可同步的插件数据'])
  })

  it('omits success when everything was skipped', () => {
    const result = summarizePluginSync({
      synced: [],
      skipped: [{ dir: 'my#data', reason: '同步失败: 文件被占用' }],
    })
    expect(result.success).toBeUndefined()
    expect(result.warnings).toEqual(['my#data: 同步失败: 文件被占用'])
  })
})
