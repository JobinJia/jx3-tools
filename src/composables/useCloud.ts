import type { CloudBatchUploadReport, CloudConfig, CloudDownloadReport, CloudProgress, CloudRoleEntry } from '@/types'
import { listen } from '@tauri-apps/api/event'
import { useMessage } from 'naive-ui'
import { ref } from 'vue'
import { cloudService } from '@/services'

/** 进度百分比（纯函数，便于测试） */
export function cloudProgressPercent(progress: CloudProgress | null): number {
  if (!progress || progress.total <= 0)
    return 0
  return Math.round((progress.current / progress.total) * 100)
}

/** 字节数格式化（纯函数，便于测试） */
export function formatBytes(bytes: number): string {
  if (bytes < 1024)
    return `${bytes} B`
  if (bytes < 1024 * 1024)
    return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

function skippedToWarnings(skipped: { dir: string, reason: string }[]): string[] {
  return skipped.map(item => `${item.dir}: ${item.reason}`)
}

/** 批量上传结果 → 提示文案（纯函数，便于测试） */
export function summarizeCloudBatchUpload(report: CloudBatchUploadReport): { success: string, warnings: string[] } {
  const warnings: string[] = []
  const withoutPlugins = report.uploaded.filter(role => role.pluginDirs.length === 0).length
  if (withoutPlugins > 0)
    warnings.push(`${withoutPlugins} 个角色未包含插件配置（未用插件登录过）`)
  warnings.push(...skippedToWarnings(report.failed))
  return { success: `已上传 ${report.uploaded.length} 个角色到云端`, warnings }
}

/** 下载结果 → 提示文案（纯函数，便于测试） */
export function summarizeCloudDownload(report: CloudDownloadReport): { success: string, warnings: string[] } {
  const plugins = report.pluginDirs.length > 0
    ? `，插件配置已落位（${report.pluginDirs.join('、')}）`
    : ''
  return {
    success: `已从云端同步：键位已就位${plugins}`,
    warnings: skippedToWarnings(report.skipped),
  }
}

// 模块级单例状态（所有组件共享）
const config = ref<CloudConfig | null>(null)
const configLoaded = ref(false)
const roles = ref<CloudRoleEntry[]>([])
const testing = ref(false)
const saving = ref(false)
const listing = ref(false)
const uploading = ref(false)
const downloading = ref(false)
const progress = ref<CloudProgress | null>(null)

export function useCloud() {
  const message = useMessage()

  function errorText(error: unknown): string {
    return error instanceof Error ? error.message : String(error)
  }

  /** 包裹一次云操作：期间监听 cloud://progress 驱动进度条，结束后清理 */
  async function withProgress<T>(label: string, fn: () => Promise<T>): Promise<T> {
    progress.value = { phase: '', current: 0, total: 0, label }
    const unlisten = await listen<CloudProgress>('cloud://progress', (e) => {
      progress.value = e.payload
    })
    try {
      return await fn()
    } finally {
      unlisten()
      progress.value = null
    }
  }

  /** 读取已保存的账号配置（应用启动后首次打开弹窗时调用） */
  async function loadConfig() {
    if (configLoaded.value)
      return
    try {
      config.value = await cloudService.getCloudConfig()
      configLoaded.value = true
    } catch (error: unknown) {
      message.error(`读取云同步配置失败: ${errorText(error)}`)
      console.error(error)
    }
  }

  /** 保存前用给定配置测试连通性 */
  async function testConnection(form: CloudConfig): Promise<boolean> {
    testing.value = true
    try {
      await cloudService.testCloudConnection(form)
      message.success('连接成功，账号可用')
      return true
    } catch (error: unknown) {
      message.error(`连接失败: ${errorText(error)}`)
      console.error(error)
      return false
    } finally {
      testing.value = false
    }
  }

  async function saveConfig(form: CloudConfig): Promise<boolean> {
    saving.value = true
    try {
      await cloudService.saveCloudConfig(form)
      config.value = { ...form }
      message.success('连接验证通过，云同步账号已保存')
      return true
    } catch (error: unknown) {
      message.error(`保存失败: ${errorText(error)}`)
      console.error(error)
      return false
    } finally {
      saving.value = false
    }
  }

  async function refreshRoles() {
    listing.value = true
    try {
      roles.value = await cloudService.listRoles()
    } catch (error: unknown) {
      message.error(`获取云端列表失败: ${errorText(error)}`)
      console.error(error)
    } finally {
      listing.value = false
    }
  }

  /** 批量上传 userdata 下所有角色（键位 + 插件配置）并刷新云端列表 */
  async function uploadAll(userdataPath: string): Promise<boolean> {
    uploading.value = true
    try {
      const report = await withProgress('准备上传…', () => cloudService.uploadAll(userdataPath))
      const { success, warnings } = summarizeCloudBatchUpload(report)
      message.success(success)
      for (const warning of warnings)
        message.warning(warning)
      await refreshRoles()
      return true
    } catch (error: unknown) {
      message.error(`上传失败: ${errorText(error)}`)
      console.error(error)
      return false
    } finally {
      uploading.value = false
    }
  }

  /** 把云端角色同步到本地目标角色 */
  async function downloadRole(key: string, targetPath: string): Promise<boolean> {
    downloading.value = true
    try {
      const report = await withProgress('准备下载…', () => cloudService.downloadRole(key, targetPath))
      const { success, warnings } = summarizeCloudDownload(report)
      message.success(success)
      for (const warning of warnings)
        message.warning(warning)
      return true
    } catch (error: unknown) {
      message.error(`下载失败: ${errorText(error)}`)
      console.error(error)
      return false
    } finally {
      downloading.value = false
    }
  }

  return {
    config,
    roles,
    testing,
    saving,
    listing,
    uploading,
    downloading,
    progress,
    loadConfig,
    testConnection,
    saveConfig,
    refreshRoles,
    uploadAll,
    downloadRole,
  }
}
