<script setup lang="ts">
import type { HotkeyConfig, WindowInfo } from '@/types'
import { useMessage } from 'naive-ui'
import { storeToRefs } from 'pinia'
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import { hotkeyService } from '@/services'
import { useHotkeyStore } from '@/stores/hotkey'

const message = useMessage()
const hotkeyStore = useHotkeyStore()
const { config, status, loading, saving } = storeToRefs(hotkeyStore)

const formValue = reactive<HotkeyConfig>({
  triggerKey: '',
  intervalMs: 1000,
  startHotkey: 'F11',
  stopHotkey: 'F12',
  keyMode: 'global',
  targetWindow: null,
})

// 窗口列表相关
const windowList = ref<WindowInfo[]>([])
const windowFilter = ref('')
const windowLoading = ref(false)
const refreshTimer = ref<number | null>(null)
const fetchRequestId = ref(0) // Track current fetch to ignore stale results

// 平台检测
const isWindows = computed(() => navigator.platform.toLowerCase().includes('win'))
const isWindowMode = computed(() => formValue.keyMode === 'window')

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

// 获取窗口列表
async function fetchWindows() {
  if (!isWindows.value)
    return

  // Increment request ID to track this fetch
  const currentRequestId = ++fetchRequestId.value

  windowLoading.value = true
  try {
    const result = await hotkeyService.listWindows(windowFilter.value || undefined)
    // Only update if this is still the latest request (not stale)
    if (currentRequestId === fetchRequestId.value) {
      windowList.value = result
    }
  } catch (error) {
    // Only log error if this is still the latest request
    if (currentRequestId === fetchRequestId.value) {
      console.error('获取窗口列表失败:', error)
    }
  } finally {
    // Only clear loading if this is still the latest request
    if (currentRequestId === fetchRequestId.value) {
      windowLoading.value = false
    }
  }
}

// 选择窗口
function handleWindowSelect(hwnd: number | null) {
  if (hwnd === null) {
    formValue.targetWindow = null
    return
  }
  const win = windowList.value.find(w => w.hwnd === hwnd)
  if (win) {
    formValue.targetWindow = {
      hwnd: win.hwnd,
      title: win.title,
      className: win.className,
      processName: win.processName,
    }
  }
}

// 窗口选项
const windowOptions = computed(() =>
  windowList.value.map(w => ({
    label: w.displayName,
    value: w.hwnd,
  })),
)

// 监听模式切换
watch(isWindowMode, (newVal) => {
  if (newVal && isWindows.value) {
    fetchWindows()
    // 启动自动刷新（每 5 秒）
    refreshTimer.value = window.setInterval(fetchWindows, 5000)
  } else {
    // 清理定时器
    if (refreshTimer.value) {
      clearInterval(refreshTimer.value)
      refreshTimer.value = null
    }
    // Invalidate any pending fetch requests
    fetchRequestId.value++
    windowLoading.value = false
  }
})

async function loadInitialData() {
  try {
    await hotkeyStore.init()
    // 如果是窗口模式，加载窗口列表
    if (formValue.keyMode === 'window' && isWindows.value) {
      fetchWindows()
      refreshTimer.value = window.setInterval(fetchWindows, 5000)
    }
  } catch (error) {
    console.error('加载按键配置失败:', error)
    message.error('加载按键配置失败')
  }
}

async function saveConfig() {
  try {
    await hotkeyStore.saveConfig({ ...formValue })
    message.success('配置已保存，按开始热键即可执行')
  } catch (error: unknown) {
    console.error('保存按键配置失败:', error)
    let msg = '保存失败，请检查输入'
    if (typeof error === 'string') {
      msg = error
    } else if (error instanceof Error) {
      msg = error.message
    } else if (error && typeof error === 'object' && 'message' in error) {
      msg = String((error as { message: unknown }).message)
    }
    message.error(msg)
  }
}

onMounted(() => {
  void loadInitialData()
})

onUnmounted(() => {
  void hotkeyStore.disposeListener()
  if (refreshTimer.value) {
    clearInterval(refreshTimer.value)
    refreshTimer.value = null
  }
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
          <n-form-item label="按键模式">
            <n-space align="center">
              <n-switch
                :value="formValue.keyMode === 'window'"
                :disabled="!isWindows"
                @update:value="(val: boolean) => formValue.keyMode = val ? 'window' : 'global'"
              >
                <template #checked>
                  窗口
                </template>
                <template #unchecked>
                  全局
                </template>
              </n-switch>
              <n-text v-if="!isWindows" depth="3">
                (窗口模式仅支持 Windows)
              </n-text>
            </n-space>
          </n-form-item>
          <n-form-item v-if="isWindowMode && isWindows" label="目标窗口">
            <n-space vertical style="width: 100%">
              <n-input-group>
                <n-input
                  v-model:value="windowFilter"
                  placeholder="输入关键词过滤窗口..."
                  clearable
                  @update:value="fetchWindows"
                />
                <n-button :loading="windowLoading" @click="fetchWindows">
                  刷新
                </n-button>
              </n-input-group>
              <n-select
                :value="formValue.targetWindow?.hwnd ?? null"
                :options="windowOptions"
                placeholder="选择目标窗口"
                filterable
                clearable
                :loading="windowLoading"
                @update:value="handleWindowSelect"
              />
              <n-text v-if="formValue.targetWindow" depth="3">
                已选择: {{ formValue.targetWindow.title }}
              </n-text>
            </n-space>
          </n-form-item>
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
            <n-input-number v-model:value="formValue.intervalMs" :min="20" :max="60000" :step="50" />
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
        <n-alert v-if="status.lastError" type="error" title="错误" class="mt-3" :bordered="false">
          {{ status.lastError }}
        </n-alert>
        <n-alert v-if="isWindowMode && isWindows" type="warning" class="mt-3" :bordered="false">
          <p><b>窗口模式说明：</b></p>
          <p>1. 按键将发送到指定窗口，即使窗口不在前台也能接收。</p>
          <p>2. 部分游戏或应用可能会屏蔽此方式的按键。</p>
          <p>3. 如果目标窗口关闭，任务会自动停止。</p>
        </n-alert>
        <n-alert type="info" class="mt-3" :bordered="false">
          <p>
            1. 保存后即可使用 <b>{{ formValue.startHotkey || '开始热键' }}</b> / <b>{{ formValue.stopHotkey || '结束热键' }}</b>
            控制任务（Windows 与 macOS 均支持）。
          </p>
          <p>2. 软件最小化或在后台时同样生效，请避免与系统或其他软件热键冲突；macOS 需在"系统设置 → 隐私与安全性 → 辅助功能"中允许应用控制键盘。</p>
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
