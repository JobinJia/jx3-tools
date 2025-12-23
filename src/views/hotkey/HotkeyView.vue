<script setup lang="ts">
import type { HotkeyConfig } from '@/types'
import { useMessage } from 'naive-ui'
import { storeToRefs } from 'pinia'
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
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

// 输入框焦点状态
const triggerKeyFocused = ref(false)
const startHotkeyFocused = ref(false)
const stopHotkeyFocused = ref(false)

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

// 将 KeyboardEvent.key 转换为按键名称
function keyEventToKeyName(e: KeyboardEvent): string {
  const key = e.key

  // 特殊键映射
  const keyMap: Record<string, string> = {
    ' ': 'Space',
    'ArrowUp': 'Up',
    'ArrowDown': 'Down',
    'ArrowLeft': 'Left',
    'ArrowRight': 'Right',
    'Control': 'Ctrl',
    'Meta': navigator.platform.includes('Mac') ? 'Cmd' : 'Win',
  }

  if (keyMap[key])
    return keyMap[key]

  // F1-F20 保持原样
  if (/^F\d+$/.test(key))
    return key

  // 单个字符转大写
  if (key.length === 1)
    return key.toUpperCase()

  // 其他按键首字母大写
  return key.charAt(0).toUpperCase() + key.slice(1)
}

// 构建组合键字符串（用于开始/结束热键）
function buildHotkeyString(e: KeyboardEvent): string {
  const parts: string[] = []

  if (e.ctrlKey)
    parts.push('Ctrl')
  if (e.altKey)
    parts.push('Alt')
  if (e.shiftKey)
    parts.push('Shift')
  if (e.metaKey)
    parts.push(navigator.platform.includes('Mac') ? 'Cmd' : 'Win')

  const key = e.key

  // 忽略单独的修饰键
  if (['Control', 'Alt', 'Shift', 'Meta'].includes(key))
    return ''

  parts.push(keyEventToKeyName(e))
  return parts.join('+')
}

// 处理触发按键的键盘事件（只捕获单个按键）
function handleTriggerKeyDown(e: KeyboardEvent) {
  e.preventDefault()
  e.stopPropagation()

  // 忽略单独的修饰键
  if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key))
    return

  formValue.triggerKey = keyEventToKeyName(e)
  triggerKeyFocused.value = false
  ;(e.target as HTMLInputElement)?.blur()
}

// 处理开始热键的键盘事件（支持组合键）
function handleStartHotkeyKeyDown(e: KeyboardEvent) {
  e.preventDefault()
  e.stopPropagation()

  const hotkey = buildHotkeyString(e)
  if (!hotkey)
    return

  formValue.startHotkey = hotkey
  startHotkeyFocused.value = false
  ;(e.target as HTMLInputElement)?.blur()
}

// 处理结束热键的键盘事件（支持组合键）
function handleStopHotkeyKeyDown(e: KeyboardEvent) {
  e.preventDefault()
  e.stopPropagation()

  const hotkey = buildHotkeyString(e)
  if (!hotkey)
    return

  formValue.stopHotkey = hotkey
  stopHotkeyFocused.value = false
  ;(e.target as HTMLInputElement)?.blur()
}

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
            <n-input
              :value="triggerKeyFocused ? '请按下按键...' : formValue.triggerKey"
              :placeholder="triggerKeyFocused ? '请按下按键...' : '点击后按下按键'"
              readonly
              :status="triggerKeyFocused ? 'warning' : undefined"
              @focus="triggerKeyFocused = true"
              @blur="triggerKeyFocused = false"
              @keydown="handleTriggerKeyDown"
            />
          </n-form-item>
          <n-form-item label="触发频率 (毫秒)">
            <n-input-number v-model:value="formValue.intervalMs" :min="20" :step="50" />
          </n-form-item>
          <n-form-item label="开始热键">
            <n-input
              :value="startHotkeyFocused ? '请按下按键...' : formValue.startHotkey"
              :placeholder="startHotkeyFocused ? '请按下按键...' : '点击后按下按键(支持组合键)'"
              readonly
              :status="startHotkeyFocused ? 'warning' : undefined"
              @focus="startHotkeyFocused = true"
              @blur="startHotkeyFocused = false"
              @keydown="handleStartHotkeyKeyDown"
            />
          </n-form-item>
          <n-form-item label="结束热键">
            <n-input
              :value="stopHotkeyFocused ? '请按下按键...' : formValue.stopHotkey"
              :placeholder="stopHotkeyFocused ? '请按下按键...' : '点击后按下按键(支持组合键)'"
              readonly
              :status="stopHotkeyFocused ? 'warning' : undefined"
              @focus="stopHotkeyFocused = true"
              @blur="stopHotkeyFocused = false"
              @keydown="handleStopHotkeyKeyDown"
            />
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
          <p>3. 点击输入框后按下键盘按键即可设置；支持字母、数字、功能键、小键盘、方向键、修饰键、媒体键等。</p>
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
