import type { FileEntry, UserSelect } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { useStorage } from '@vueuse/core'
import { useMessage } from 'naive-ui'
import { ref } from 'vue'
import { keyboardService } from '@/services'

export function useKeyboard() {
  const message = useMessage()
  const basePath = useStorage('keyboard-base-path', '')
  const treeData = ref<FileEntry[]>([])
  const loading = ref(false)

  /**
   * Open directory dialog to select keyboard config directory
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
   * Copy keyboard config from source to target
   */
  async function copyKeyboardConfig(userSelect: UserSelect): Promise<boolean> {
    if (!basePath.value) {
      message.error('请先选择目录')
      return false
    }

    // Validate source and target are not empty
    if (!userSelect.sourcePath || !userSelect.targetPath) {
      message.error('请选择源角色和目标角色')
      return false
    }

    // Prevent copying to same path
    if (userSelect.sourcePath === userSelect.targetPath) {
      message.error('源角色和目标角色不能相同')
      return false
    }

    const params = {
      source_path: `${basePath.value}/${userSelect.sourcePath}`,
      target_path: `${basePath.value}/${userSelect.targetPath}`,
    }

    try {
      const success = await keyboardService.copySourceToTarget(params)
      if (success) {
        message.success('键位复制成功')
      } else {
        message.error('复制失败')
      }
      return success
    } catch (error: unknown) {
      const errorMsg = error instanceof Error ? error.message : String(error)
      message.error(`复制失败: ${errorMsg}`)
      console.error(error)
      return false
    }
  }

  return {
    basePath,
    treeData,
    loading,
    selectDirectory,
    resetDirectory,
    loadTree,
    copyKeyboardConfig,
  }
}
