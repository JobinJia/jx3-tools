import type { HotkeyConfig, HotkeyStatus, WindowInfo } from '@/types'
import { invoke } from '@tauri-apps/api/core'

export const hotkeyService = {
  /**
   * Get current hotkey configuration
   */
  async getConfig(): Promise<HotkeyConfig> {
    return invoke<HotkeyConfig>('get_hotkey_config')
  },

  /**
   * Get current hotkey runtime status
   */
  async getStatus(): Promise<HotkeyStatus> {
    return invoke<HotkeyStatus>('get_hotkey_status')
  },

  /**
   * Save hotkey configuration
   */
  async saveConfig(config: HotkeyConfig): Promise<HotkeyConfig> {
    return invoke<HotkeyConfig>('save_hotkey_config', { config })
  },

  /**
   * Stop running hotkey automation task
   */
  async stopTask(): Promise<void> {
    return invoke<void>('stop_hotkey_task')
  },

  /**
   * Get list of visible windows (Windows only)
   */
  async listWindows(filter?: string): Promise<WindowInfo[]> {
    return invoke<WindowInfo[]>('list_windows', { filter: filter || null })
  },

  /**
   * Check if a window is still valid
   */
  async checkWindowValid(hwnd: number): Promise<boolean> {
    return invoke<boolean>('check_window_valid', { hwnd })
  },

  /**
   * Install the Interception keyboard driver (mouse filter is removed
   * right after install; reboot required). Windows only.
   */
  async installDriver(): Promise<HotkeyStatus> {
    return invoke<HotkeyStatus>('install_hotkey_driver')
  },

  /**
   * Uninstall the Interception driver (reboot required). Windows only.
   */
  async uninstallDriver(): Promise<HotkeyStatus> {
    return invoke<HotkeyStatus>('uninstall_hotkey_driver')
  },

  /**
   * Remove the leftover interception mouse filter (legacy full install)
   */
  async removeMouseFilter(): Promise<HotkeyStatus> {
    return invoke<HotkeyStatus>('remove_mouse_filter')
  },
}
