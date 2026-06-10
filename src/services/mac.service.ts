import type { MacInfo } from '@/types/mac'
import { invoke } from '@tauri-apps/api/core'

export const macService = {
  /**
   * Get the primary adapter's MAC info
   */
  async getMacInfo(): Promise<MacInfo> {
    return invoke<MacInfo>('get_mac_info')
  },

  /**
   * Change MAC address to a random value; resolves with the verified actual state
   */
  async randomizeMacAddress(): Promise<MacInfo> {
    return invoke<MacInfo>('randomize_mac_address')
  },

  /**
   * Restore the original MAC address; resolves with the verified actual state
   */
  async restoreMacAddress(): Promise<MacInfo> {
    return invoke<MacInfo>('restore_mac_cmd')
  },

  /**
   * Get auto-restore on reboot setting
   */
  async getAutoRestoreSetting(): Promise<boolean> {
    return invoke<boolean>('get_auto_restore_setting')
  },

  /**
   * Set auto-restore on reboot setting
   */
  async setAutoRestoreSetting(autoRestore: boolean): Promise<void> {
    return invoke<void>('set_auto_restore_setting', { autoRestore })
  },
}
