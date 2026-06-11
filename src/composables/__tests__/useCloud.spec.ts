import { describe, expect, it } from 'vitest'
import { formatBytes, summarizeCloudDownload, summarizeCloudUpload } from '../useCloud'

describe('formatBytes', () => {
  it('formats bytes / KB / MB', () => {
    expect(formatBytes(512)).toBe('512 B')
    expect(formatBytes(2048)).toBe('2.0 KB')
    expect(formatBytes(1536 * 1024)).toBe('1.5 MB')
  })
})

describe('summarizeCloudUpload', () => {
  it('mentions key, sizes and plugin dirs on full upload', () => {
    const result = summarizeCloudUpload({
      key: '梦江南/落梅听风雪',
      keybindingSize: 2048,
      pluginsSize: 1024,
      pluginDirs: ['my#data', 'SG#data'],
      skipped: [],
    })
    expect(result.success).toBe('已上传「梦江南/落梅听风雪」：键位 2.0 KB，插件配置 1.0 KB（my#data、SG#data）')
    expect(result.warnings).toEqual([])
  })

  it('notes missing plugins and surfaces skip reasons', () => {
    const result = summarizeCloudUpload({
      key: '梦江南/无插件角色',
      keybindingSize: 512,
      pluginsSize: 0,
      pluginDirs: [],
      skipped: [{ dir: '插件配置', reason: '未找到该角色的插件数据' }],
    })
    expect(result.success).toBe('已上传「梦江南/无插件角色」：键位 512 B（未包含插件配置）')
    expect(result.warnings).toEqual(['插件配置: 未找到该角色的插件数据'])
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
