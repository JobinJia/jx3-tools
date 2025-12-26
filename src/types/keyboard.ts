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

/** 常用键位模板 */
export interface KeyboardTemplate {
  id: string
  name: string
  description: string
  sourcePath: string
  characterName: string
  createdAt: number
}
