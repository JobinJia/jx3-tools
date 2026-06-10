import type { CopyParams, FileEntry, PluginSyncReport } from '@/types'
import { invoke } from '@tauri-apps/api/core'

export const keyboardService = {
  /**
   * List directory contents for keyboard config selection
   */
  async listDirectoryContents(path: string): Promise<FileEntry[]> {
    return invoke<FileEntry[]>('list_directory_contents', { path })
  },

  /**
   * Copy keyboard config from source to target directory
   */
  async copySourceToTarget(params: CopyParams): Promise<boolean> {
    return invoke<boolean>('cp_source_to_target', { params })
  },

  /**
   * Sync plugin config (interface/*#data) from source role to target role
   */
  async syncPluginConfig(params: CopyParams): Promise<PluginSyncReport> {
    return invoke<PluginSyncReport>('sync_plugin_config', { params })
  },

  /**
   * Open folder in system file explorer
   */
  async openFolder(path: string): Promise<void> {
    return invoke<void>('open_folder', { path })
  },
}
