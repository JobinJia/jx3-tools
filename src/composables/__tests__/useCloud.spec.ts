import { describe, expect, it } from 'vitest'
import { formatBytes, summarizeCloudBatchUpload, summarizeCloudDownload } from '../useCloud'

describe('formatBytes', () => {
  it('formats bytes / KB / MB', () => {
    expect(formatBytes(512)).toBe('512 B')
    expect(formatBytes(2048)).toBe('2.0 KB')
    expect(formatBytes(1536 * 1024)).toBe('1.5 MB')
  })
})

describe('summarizeCloudBatchUpload', () => {
  const role = (key: string, pluginDirs: string[]) => ({
    key,
    keybindingSize: 1024,
    pluginsSize: pluginDirs.length > 0 ? 512 : 0,
    pluginDirs,
    skipped: [],
  })

  it('counts uploaded roles and roles without plugin configs', () => {
    const result = summarizeCloudBatchUpload({
      uploaded: [role('梦江南/甲', ['my#data']), role('梦江南/乙', [])],
      failed: [{ dir: '丙', reason: '角色目录不存在' }],
    })
    expect(result.success).toBe('已上传 2 个角色到云端')
    expect(result.warnings).toEqual([
      '1 个角色未包含插件配置（未用插件登录过）',
      '丙: 角色目录不存在',
    ])
  })

  it('clean batch yields success only', () => {
    const result = summarizeCloudBatchUpload({
      uploaded: [role('梦江南/甲', ['my#data']), role('梦江南/乙', ['lm#data'])],
      failed: [],
    })
    expect(result.success).toBe('已上传 2 个角色到云端')
    expect(result.warnings).toEqual([])
  })
})

describe('summarizeCloudDownload', () => {
  it('mentions keybinding and plugin dirs', () => {
    const result = summarizeCloudDownload({
      keybindingApplied: true,
      pluginDirs: ['my#data'],
      skipped: [],
    })
    expect(result.success).toBe('已从云端同步：键位已就位，插件配置已落位（my#data）')
    expect(result.warnings).toEqual([])
  })

  it('keybinding only with warnings for skipped dirs', () => {
    const result = summarizeCloudDownload({
      keybindingApplied: true,
      pluginDirs: [],
      skipped: [{ dir: 'lm#data', reason: '本机未安装该插件（无对应数据目录）' }],
    })
    expect(result.success).toBe('已从云端同步：键位已就位')
    expect(result.warnings).toEqual(['lm#data: 本机未安装该插件（无对应数据目录）'])
  })
})
