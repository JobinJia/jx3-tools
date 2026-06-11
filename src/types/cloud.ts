import type { PluginSyncSkippedItem } from './keyboard'

/** 云同步账号配置（WebDAV） */
export interface CloudConfig {
  serverUrl: string
  username: string
  appPassword: string
}

/** 云端角色存档条目（manifest.json） */
export interface CloudRoleEntry {
  key: string
  name: string
  server: string
  uploadedAt: number
  device: string
  keybindingFile: string
  pluginsFile: string | null
  keybindingSize: number
  pluginsSize: number
}

/** 上传结果（cloud_upload_role 返回值） */
export interface CloudUploadReport {
  key: string
  keybindingSize: number
  pluginsSize: number
  pluginDirs: string[]
  skipped: PluginSyncSkippedItem[]
}

/** 下载结果（cloud_download_role 返回值） */
export interface CloudDownloadReport {
  keybindingApplied: boolean
  pluginDirs: string[]
  skipped: PluginSyncSkippedItem[]
}
