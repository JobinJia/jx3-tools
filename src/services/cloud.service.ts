import type {
  CloudConfig,
  CloudDownloadReport,
  CloudRoleEntry,
  CloudUploadReport,
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
   * Pack and upload a role (keybinding + plugin configs) to the cloud
   */
  async uploadRole(rolePath: string): Promise<CloudUploadReport> {
    return invoke<CloudUploadReport>('cloud_upload_role', { rolePath })
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
