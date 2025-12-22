<script setup lang="ts">
import type { HotkeyConfig } from '@/types'
import { useMessage } from 'naive-ui'
import { storeToRefs } from 'pinia'
import { computed, onMounted, onUnmounted, reactive, watch } from 'vue'
import { useHotkeyStore } from '@/stores/hotkey'

const message = useMessage()
const hotkeyStore = useHotkeyStore()
const { config, status, loading, saving } = storeToRefs(hotkeyStore)

const formValue = reactive<HotkeyConfig>({
  triggerKey: '',
  intervalMs: 1000,
  startHotkey: 'F11',
  stopHotkey: 'F12',
})

watch(
  config,
  (value) => {
    if (value)
      Object.assign(formValue, value)
  },
  { immediate: true },
)

const statusText = computed(() => (status.value.running ? '运行中' : '已停止'))
const statusType = computed(() => {
  if (status.value.running)
    return 'success'
  if (!status.value.registered)
    return 'warning'
  return 'default'
})

async function loadInitialData() {
  try {
    await hotkeyStore.init()
  } catch (error) {
    console.error('加载按键配置失败:', error)
    message.error('加载按键配置失败')
  }
}

async function saveConfig() {
  try {
    await hotkeyStore.saveConfig({ ...formValue })
    message.success('配置已保存，按开始热键即可执行')
  } catch (error: any) {
    console.error('保存按键配置失败:', error)
    const msg = typeof error === 'string' ? error : '保存失败，请检查输入'
    message.error(msg)
  }
}

onMounted(() => {
  void loadInitialData()
})

onUnmounted(() => {
  void hotkeyStore.disposeListener()
})
</script>

<template>
  <div class="hotkey-container">
    <n-card title="自动按键设置" :bordered="false">
      <n-spin :show="loading">
        <n-form
          v-if="config" label-placement="left" :label-width="120" :model="formValue" size="medium"
          class="hotkey-form"
        >
          <n-form-item label="触发按键">
            <n-input v-model:value="formValue.triggerKey" placeholder="例如：1、Q、F6" maxlength="10" clearable />
          </n-form-item>
          <n-form-item label="触发频率 (毫秒)">
            <n-input-number v-model:value="formValue.intervalMs" :min="20" :step="50" />
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
              <n-button :disabled="!status.running" @click="hotkeyStore.stopTask">
                停止任务
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
            1. 保存后即可使用 <b>{{ formValue.startHotkey || '开始热键' }}</b> / <b>{{ formValue.stopHotkey || '结束热键' }}</b>
            控制任务（Windows 与 macOS 均支持）。
          </p>
          <p>2. 软件最小化或在后台时同样生效，请避免与系统或其他软件热键冲突；macOS 需在“系统设置 → 隐私与安全性 → 辅助功能”中允许应用控制键盘。</p>
          <p>3. 触发按键支持字母、数字、功能键（Windows 可至 F24，macOS 至 F20）以及常用操作键（空格、方向键等）。</p>
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
