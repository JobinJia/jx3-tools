/** Hotkey configuration stored in backend */
export interface HotkeyConfig {
  triggerKey: string
  intervalMs: number
  startHotkey: string
  stopHotkey: string
}

/** Hotkey runtime status */
export interface HotkeyStatus {
  running: boolean
  registered: boolean
  lastError: string | null
}
