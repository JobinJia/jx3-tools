import type {
  CloudBatchUploadReport,
  CloudConfig,
  CloudDownloadReport,
  CloudRoleEntry,
} from '@/types'
import { invoke } from '@tauri-apps/api/core'

export const cloudService = {
  /**
   * Load saved WebDAV account config (null when not configured)
   */
  async getCloudConfig(): Promise<CloudConfig | null> {
    return invoke<CloudConfig | null>('get_cloud_config')
  },

  /**
   * Validate and persist WebDAV account config
   */
  async saveCloudConfig(config: CloudConfig): Promise<void> {
    return invoke<void>('save_cloud_config', { config })
  },

  /**
   * Test connectivity/credentials with the given (unsaved) config
   */
  async testCloudConnection(config: CloudConfig): Promise<void> {
    return invoke<void>('test_cloud_connection', { config })
  },

  /**
   * Pack and upload all roles under userdata (keybinding + plugin configs)
   */
  async uploadAll(userdataPath: string): Promise<CloudBatchUploadReport> {
    return invoke<CloudBatchUploadReport>('cloud_upload_all', { userdataPath })
  },

  /**
   * List roles stored in the cloud (from manifest)
   */
  async listRoles(): Promise<CloudRoleEntry[]> {
    return invoke<CloudRoleEntry[]>('cloud_list_roles')
  },

  /**
   * Download a cloud role onto a local target role
   */
  async downloadRole(key: string, targetPath: string): Promise<CloudDownloadReport> {
    return invoke<CloudDownloadReport>('cloud_download_role', { key, targetPath })
  },
}
