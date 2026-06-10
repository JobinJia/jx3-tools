import type { MacInfo } from '@/types/mac'
import { useMessage } from 'naive-ui'
import { computed, ref } from 'vue'
import { macService } from '@/services'

export function useMac() {
  const message = useMessage()

  // Backend returns the verified actual adapter state — never display
  // an optimistic value that the driver may have silently rejected
  const info = ref<MacInfo | null>(null)
  const autoRestoreEnabled = ref(false)
  const loading = ref(false)
  const changing = ref(false)
  const restoring = ref(false)
  const error = ref<string | null>(null)

  const adapterName = computed(() => info.value?.adapterName ?? '')
  const currentAddress = computed(() => info.value?.currentMac ?? '')
  const originalAddress = computed(() => info.value?.permanentMac ?? '')
  const isChanged = computed(() => info.value?.isModified ?? false)

  /**
   * Fetch current MAC info and auto-restore setting from backend
   */
  async function fetchMacAddress() {
    loading.value = true
    error.value = null

    try {
      info.value = await macService.getMacInfo()
    } catch (e: unknown) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      error.value = `获取MAC地址失败: ${errorMsg}`
      message.error('获取MAC地址失败')
      info.value = null
    } finally {
      loading.value = false
    }

    try {
      autoRestoreEnabled.value = await macService.getAutoRestoreSetting()
    } catch {
      autoRestoreEnabled.value = false
    }
  }

  /**
   * Change MAC address to a random value (verified by the backend)
   */
  async function changeMacAddress() {
    changing.value = true
    error.value = null

    try {
      info.value = await macService.randomizeMacAddress()
      message.success(`MAC地址已修改为 ${info.value.currentMac}`)
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
      info.value = await macService.restoreMacAddress()
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
    adapterName,
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
