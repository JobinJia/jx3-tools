/** 按键发送模式 */
export type KeyMode = 'global' | 'window'

/** 目标窗口信息 */
export interface TargetWindow {
  hwnd: number
  title: string
  className: string
  processName: string
}

/** 窗口列表项（用于下拉选择） */
export interface WindowInfo {
  hwnd: number
  title: string
  className: string
  processName: string
  displayName: string
}

/** Hotkey configuration stored in backend */
export interface HotkeyConfig {
  triggerKey: string
  intervalMs: number
  startHotkey: string
  stopHotkey: string
  keyMode: KeyMode
  targetWindow: TargetWindow | null
}

/** Hotkey runtime status */
export interface HotkeyStatus {
  running: boolean
  registered: boolean
  lastError: string | null
}
