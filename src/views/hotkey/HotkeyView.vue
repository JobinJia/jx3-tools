<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useMessage } from 'naive-ui'
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'

interface HotkeyConfig {
  triggerKey: string
  intervalMs: number
  startHotkey: string
  stopHotkey: string
}

interface HotkeyStatus {
  running: boolean
  lastError: string | null
}

const message = useMessage()
const formValue = reactive<HotkeyConfig>({
  triggerKey: '',
  intervalMs: 1000,
  startHotkey: 'F11',
  stopHotkey: 'F12',
})
const status = ref<HotkeyStatus>({ running: false, lastError: null })
const loading = ref(false)
const saving = ref(false)
let unlisten: null | (() => void) = null

const statusText = computed(() => (status.value.running ? '运行中' : '已停止'))
const statusType = computed(() => (status.value.running ? 'success' : 'default'))

async function fetchConfig() {
  loading.value = true
  try {
    const config = await invoke<HotkeyConfig>('get_hotkey_config')
    Object.assign(formValue, config)
    const currentStatus = await invoke<HotkeyStatus>('get_hotkey_status')
    status.value = currentStatus
  } catch (error) {
    console.error('加载按键配置失败:', error)
    message.error('加载按键配置失败')
  } finally {
    loading.value = false
  }
}

async function saveConfig() {
  saving.value = true
  try {
    const payload = await invoke<HotkeyConfig>('save_hotkey_config', {
      config: { ...formValue },
    })
    Object.assign(formValue, payload)
    message.success('配置已保存，按开始热键即可执行')
  } catch (error: any) {
    console.error('保存按键配置失败:', error)
    const msg = typeof error === 'string' ? error : '保存失败，请检查输入'
    message.error(msg)
  } finally {
    saving.value = false
  }
}

onMounted(async () => {
  await fetchConfig()
  unlisten = await listen<HotkeyStatus>('hotkey://status', (event) => {
    status.value = event.payload
  })
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
    unlisten = null
  }
})
</script>

<template>
  <div class="hotkey-container">
    <n-card title="自动按键设置" :bordered="false">
      <n-spin :show="loading">
        <n-form
          label-placement="left"
          :label-width="120"
          :model="formValue"
          size="medium"
          class="hotkey-form"
        >
          <n-form-item label="触发按键">
            <n-input
              v-model:value="formValue.triggerKey"
              placeholder="例如：1、Q、F6"
              maxlength="10"
              clearable
            />
          </n-form-item>
          <n-form-item label="触发频率 (毫秒)">
            <n-input-number v-model:value="formValue.intervalMs" :min="10" :step="50" />
          </n-form-item>
          <n-form-item label="开始热键">
            <n-input v-model:value="formValue.startHotkey" placeholder="例如：F11 或 Ctrl+Alt+S" clearable />
          </n-form-item>
          <n-form-item label="结束热键">
            <n-input v-model:value="formValue.stopHotkey" placeholder="例如：F12 或 Ctrl+Alt+D" clearable />
          </n-form-item>
          <n-form-item>
            <n-space>
              <n-button type="primary" :loading="saving" @click="saveConfig">
                保存配置
              </n-button>
              <n-tag :type="statusType">
                当前状态：{{ statusText }}
              </n-tag>
            </n-space>
          </n-form-item>
        </n-form>
        <n-alert v-if="status.lastError" type="warning" title="执行失败" class="mt-3" :bordered="false">
          {{ status.lastError }}
        </n-alert>
        <n-alert type="info" class="mt-3" :bordered="false">
          <p>
            1. 保存后即可使用 <b>{{ formValue.startHotkey || '开始热键' }}</b>
            / <b>{{ formValue.stopHotkey || '结束热键' }}</b> 控制任务。
          </p>
          <p>2. 软件最小化或在后台时同样生效，避免与其他热键冲突。</p>
          <p>3. 触发按键支持字母、数字、功能键（F1~F24）以及常用操作键（空格、方向键等）。</p>
        </n-alert>
      </n-spin>
    </n-card>
  </div>
</template>

<style scoped>
.hotkey-container {
  padding: 20px;
  height: 100%;
  box-sizing: border-box;
}

.hotkey-form {
  max-width: 520px;
}

.mt-3 {
  margin-top: 12px;
}
</style>
