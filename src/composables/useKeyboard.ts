import type { CopyParams, FileEntry, KeyboardTemplate, PluginSyncReport, UserSelect } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { useStorage } from '@vueuse/core'
import { useMessage } from 'naive-ui'
import { ref } from 'vue'
import { keyboardService } from '@/services'

// 模块级单例状态（所有组件共享）
const basePath = useStorage('keyboard-base-path', '')
const treeData = ref<FileEntry[]>([])
const loading = ref(false)
const copyLoading = ref(false)
const templates = useStorage<KeyboardTemplate[]>('keyboard-templates', [])
// 复制键位时是否同时同步插件配置（interface/*#data 下的茗伊/枫影等）
const syncPluginEnabled = useStorage('keyboard-sync-plugin', false)

/** 把插件同步结果整理成提示文案（纯函数，便于测试） */
export function summarizePluginSync(report: PluginSyncReport): { success?: string, warnings: string[] } {
  const warnings = report.skipped.map(item => `${item.dir}: ${item.reason}`)
  if (report.synced.length === 0 && warnings.length === 0)
    return { warnings: ['未发现可同步的插件数据'] }
  if (report.synced.length === 0)
    return { warnings }
  return { success: `插件配置已同步: ${report.synced.join('、')}`, warnings }
}

export function useKeyboard() {
  const message = useMessage()

  /**
   * Open directory dialog to select keyboard config directory (首次选择)
   */
  async function selectDirectory() {
    if (basePath.value)
      return

    const path = await open({
      multiple: false,
      directory: true,
    })

    if (path) {
      basePath.value = path
    }
  }

  /**
   * Change to a different directory (允许重新选择)
   */
  async function changeDirectory() {
    const path = await open({
      multiple: false,
      directory: true,
      defaultPath: basePath.value || undefined,
    })

    if (path) {
      basePath.value = path
      treeData.value = []
      await loadTree()
    }
  }

  /**
   * Reset the selected directory
   */
  function resetDirectory() {
    basePath.value = ''
    treeData.value = []
  }

  /**
   * Load directory tree from backend
   */
  async function loadTree() {
    if (!basePath.value)
      return

    loading.value = true
    try {
      treeData.value = await keyboardService.listDirectoryContents(basePath.value)
    } catch (error) {
      message.error('加载目录失败')
      console.error(error)
    } finally {
      loading.value = false
    }
  }

  /**
   * 键位复制成功后的插件配置同步；失败只告警，不影响已完成的键位复制
   */
  async function syncPluginAfterCopy(params: CopyParams) {
    try {
      const report = await keyboardService.syncPluginConfig(params)
      const { success, warnings } = summarizePluginSync(report)
      if (success)
        message.success(success)
      for (const warning of warnings)
        message.warning(warning)
    } catch (error: unknown) {
      const errorMsg = error instanceof Error ? error.message : String(error)
      message.warning(`插件配置同步失败（键位复制不受影响）: ${errorMsg}`)
      console.error(error)
    }
  }

  /**
   * Copy keyboard config from source to target
   */
  async function copyKeyboardConfig(userSelect: UserSelect): Promise<boolean> {
    if (!basePath.value) {
      message.error('请先选择目录')
      return false
    }

    if (!userSelect.sourcePath || !userSelect.targetPath) {
      message.error('请选择源角色和目标角色')
      return false
    }

    if (userSelect.sourcePath === userSelect.targetPath) {
      message.error('源角色和目标角色不能相同')
      return false
    }

    const params = {
      source_path: `${basePath.value}/${userSelect.sourcePath}`,
      target_path: `${basePath.value}/${userSelect.targetPath}`,
    }

    copyLoading.value = true
    try {
      const success = await keyboardService.copySourceToTarget(params)
      if (success) {
        message.success('键位复制成功')
        if (syncPluginEnabled.value)
          await syncPluginAfterCopy(params)
        await loadTree()
      } else {
        message.error('复制失败')
      }
      return success
    } catch (error: unknown) {
      const errorMsg = error instanceof Error ? error.message : String(error)
      message.error(`复制失败: ${errorMsg}`)
      console.error(error)
      return false
    } finally {
      copyLoading.value = false
    }
  }

  /**
   * Save current source as a template
   */
  function saveTemplate(name: string, description: string, sourcePath: string, characterName: string) {
    const template: KeyboardTemplate = {
      id: `template_${Date.now()}`,
      name,
      description,
      sourcePath,
      characterName,
      createdAt: Date.now(),
    }
    templates.value.push(template)
    message.success(`已保存常用键位: ${name}`)
    return template
  }

  /**
   * Delete a template by id
   */
  function deleteTemplate(id: string) {
    const idx = templates.value.findIndex(t => t.id === id)
    if (idx >= 0) {
      templates.value.splice(idx, 1)
      message.success('已删除')
    }
  }

  /**
   * Apply a template to target path
   */
  async function applyTemplate(templateId: string, targetPath: string): Promise<boolean> {
    const template = templates.value.find(t => t.id === templateId)
    if (!template) {
      message.error('模板不存在')
      return false
    }

    if (!basePath.value) {
      message.error('请先选择目录')
      return false
    }

    if (!targetPath) {
      message.error('请先选择目标角色')
      return false
    }

    if (template.sourcePath === targetPath) {
      message.error('源角色和目标角色不能相同')
      return false
    }

    const params = {
      source_path: `${basePath.value}/${template.sourcePath}`,
      target_path: `${basePath.value}/${targetPath}`,
    }

    copyLoading.value = true
    try {
      const success = await keyboardService.copySourceToTarget(params)
      if (success) {
        message.success(`已应用键位: ${template.name}`)
        if (syncPluginEnabled.value)
          await syncPluginAfterCopy(params)
        await loadTree()
      } else {
        message.error('应用失败')
      }
      return success
    } catch (error: unknown) {
      const errorMsg = error instanceof Error ? error.message : String(error)
      message.error(`应用失败: ${errorMsg}`)
      console.error(error)
      return false
    } finally {
      copyLoading.value = false
    }
  }

  return {
    basePath,
    treeData,
    loading,
    copyLoading,
    templates,
    syncPluginEnabled,
    selectDirectory,
    changeDirectory,
    resetDirectory,
    loadTree,
    copyKeyboardConfig,
    saveTemplate,
    deleteTemplate,
    applyTemplate,
  }
}
