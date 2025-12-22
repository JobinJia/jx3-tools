import type { HotkeyConfig, HotkeyStatus } from '@/types'
import { listen } from '@tauri-apps/api/event'
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { hotkeyService } from '@/services'

const STATUS_EVENT = 'hotkey://status'

export const useHotkeyStore = defineStore('hotkey', () => {
  const config = ref<HotkeyConfig | null>(null)
  const status = ref<HotkeyStatus>({ running: false, registered: false, lastError: null })
  const loading = ref(false)
  const saving = ref(false)
  let stopListener: null | (() => void) = null

  async function ensureListener() {
    if (stopListener)
      return

    stopListener = await listen<HotkeyStatus>(STATUS_EVENT, (event) => {
      status.value = event.payload
    })
  }

  async function disposeListener() {
    if (stopListener) {
      await stopListener()
      stopListener = null
    }
  }

  async function fetchConfig() {
    loading.value = true
    try {
      config.value = await hotkeyService.getConfig()
      status.value = await hotkeyService.getStatus()
    } finally {
      loading.value = false
    }
  }

  async function saveConfig(next: HotkeyConfig) {
    saving.value = true
    try {
      config.value = await hotkeyService.saveConfig(next)
    } finally {
      saving.value = false
    }
  }

  async function stopTask() {
    await hotkeyService.stopTask()
  }

  async function init() {
    await fetchConfig()
    await ensureListener()
  }

  return {
    config,
    status,
    loading,
    saving,
    init,
    fetchConfig,
    saveConfig,
    stopTask,
    disposeListener,
  }
})
