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

/** 按键驱动安装状态 */
export type DriverState = 'ready' | 'pendingReboot' | 'notInstalled'

/** Hotkey runtime status */
export interface HotkeyStatus {
  running: boolean
  registered: boolean
  lastError: string | null
  /** 按键驱动（Interception）是否就绪 */
  driverReady: boolean
  /** 驱动安装状态（已就绪 / 等待重启 / 未安装） */
  driverState: DriverState
  /** 是否残留 interception 鼠标过滤器（旧版安装包遗留，需清理） */
  mouseFilterPresent: boolean
}
