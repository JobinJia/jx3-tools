import { invoke } from '@tauri-apps/api/core'

export const macService = {
  /**
   * Get current MAC address
   */
  async getMacAddress(): Promise<string> {
    return invoke<string>('get_mac_address')
  },

  /**
   * Change MAC address to a new value
   */
  async changeMacAddress(macAddress: string): Promise<void> {
    return invoke<void>('change_mac_address', { macAddress })
  },

  /**
   * Restore original MAC address
   */
  async restoreMacAddress(): Promise<void> {
    return invoke<void>('restore_mac_cmd')
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
