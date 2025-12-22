import { useMessage } from 'naive-ui'
import { computed, ref } from 'vue'
import { macService } from '@/services'

export function useMac() {
  const message = useMessage()

  const originalAddress = ref('')
  const currentAddress = ref('')
  const autoRestoreEnabled = ref(true)
  const loading = ref(false)
  const changing = ref(false)
  const restoring = ref(false)
  const error = ref<string | null>(null)

  const isChanged = computed(() =>
    originalAddress.value !== currentAddress.value && currentAddress.value !== '',
  )

  /**
   * Generate a random locally administered unicast MAC address
   */
  function generateRandomMac(): string {
    const bytes = Array.from({ length: 6 }, () => Math.floor(Math.random() * 256))
    // Ensure first byte is unicast (LSB=0) and locally administered (second LSB=1)
    bytes[0] = (bytes[0]! | 0x02) & 0xFE
    return bytes.map(b => b.toString(16).padStart(2, '0')).join(':').toUpperCase()
  }

  /**
   * Fetch current MAC address from backend
   */
  async function fetchMacAddress() {
    loading.value = true
    error.value = null

    try {
      const address = await macService.getMacAddress()
      originalAddress.value = address
      currentAddress.value = address

      try {
        autoRestoreEnabled.value = await macService.getAutoRestoreSetting()
      } catch {
        autoRestoreEnabled.value = true
        await macService.setAutoRestoreSetting(true).catch(() => {})
      }
    } catch (e: unknown) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      error.value = `获取MAC地址失败: ${errorMsg}`
      message.error('获取MAC地址失败')
      // Keep addresses empty on error - don't use fake values
      originalAddress.value = ''
      currentAddress.value = ''
    } finally {
      loading.value = false
    }
  }

  /**
   * Change MAC address to a random value
   */
  async function changeMacAddress() {
    if (!originalAddress.value) {
      message.error('请先获取当前MAC地址')
      return
    }

    changing.value = true
    error.value = null

    try {
      const newMac = generateRandomMac()
      await macService.changeMacAddress(newMac)
      currentAddress.value = newMac
      message.success('MAC地址修改成功')
    } catch (e: unknown) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      error.value = errorMsg
      message.error(errorMsg)
    } finally {
      changing.value = false
    }
  }

  /**
   * Restore original MAC address
   */
  async function restoreMacAddress() {
    restoring.value = true
    error.value = null

    try {
      await macService.restoreMacAddress()
      const address = await macService.getMacAddress()
      currentAddress.value = address
      originalAddress.value = address
      message.success('MAC地址已还原')
    } catch (e: unknown) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      error.value = errorMsg
      message.error(errorMsg)
    } finally {
      restoring.value = false
    }
  }

  /**
   * Set auto-restore on reboot setting
   */
  async function setAutoRestore(enabled: boolean) {
    try {
      await macService.setAutoRestoreSetting(enabled)
      autoRestoreEnabled.value = enabled
      message.success(enabled ? '已开启重启自动还原' : '已关闭重启自动还原')
    } catch (e: unknown) {
      autoRestoreEnabled.value = !enabled
      const errorMsg = e instanceof Error ? e.message : String(e)
      message.error(`更新设置失败: ${errorMsg}`)
    }
  }

  return {
    originalAddress,
    currentAddress,
    autoRestoreEnabled,
    loading,
    changing,
    restoring,
    error,
    isChanged,
    fetchMacAddress,
    changeMacAddress,
    restoreMacAddress,
    setAutoRestore,
  }
}
