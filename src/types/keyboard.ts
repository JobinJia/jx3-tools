/** File/directory entry returned from backend */
export interface FileEntry {
  id: number
  name: string
  is_dir: boolean
  selected: boolean
  children?: FileEntry[]
}

/** Parameters for copying keyboard config */
export interface CopyParams {
  source_path: string
  target_path: string
}

/** User's selection state for keyboard config copy */
export interface UserSelect {
  source: string
  sourcePath: string
  target: string
  targetPath: string
}

/** 插件配置同步时跳过的数据目录及原因 */
export interface PluginSyncSkippedItem {
  dir: string
  reason: string
}

/** 插件配置同步结果（sync_plugin_config 返回值） */
export interface PluginSyncReport {
  synced: string[]
  skipped: PluginSyncSkippedItem[]
}

/** 常用键位模板 */
export interface KeyboardTemplate {
  id: string
  name: string
  description: string
  sourcePath: string
  characterName: string
  createdAt: number
}
