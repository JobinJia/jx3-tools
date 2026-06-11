<script setup lang="ts">
import type { HotkeyConfig, WindowInfo } from '@/types'
import { useMessage } from 'naive-ui'
import { storeToRefs } from 'pinia'
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import { hotkeyService } from '@/services'
import { useHotkeyStore } from '@/stores/hotkey'

const message = useMessage()
const hotkeyStore = useHotkeyStore()
const { config, status, loading, saving, driverBusy } = storeToRefs(hotkeyStore)

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

const statusText = computed(() => {
  if (status.value.running)
    return '运行中'
  if (!status.value.registered)
    return '未注册'
  return '已停止'
})
const statusClass = computed(() => {
  if (status.value.running)
    return 'running'
  if (!status.value.registered)
    return 'warn'
  return 'idle'
})

// 驱动安装状态（仅 Windows 有意义）
const driverState = computed(() => status.value.driverState)
const showDriverInstall = computed(() => isWindows.value && driverState.value === 'notInstalled')
const showDriverReboot = computed(() => isWindows.value && driverState.value === 'pendingReboot')
// 旧版安装包附带的鼠标过滤器残留（任何驱动状态下都提示清理）
const showMouseFilterWarn = computed(() => isWindows.value && status.value.mouseFilterPresent)

function errorText(error: unknown, fallback: string): string {
  if (typeof error === 'string')
    return error
  if (error instanceof Error)
    return error.message
  if (error && typeof error === 'object' && 'message' in error)
    return String((error as { message: unknown }).message)
  return fallback
}

async function handleInstallDriver() {
  try {
    await hotkeyStore.installDriver()
    message.success('驱动已安装（仅键盘过滤器），请重启电脑后使用按键功能')
  } catch (error: unknown) {
    console.error('安装按键驱动失败:', error)
    message.error(errorText(error, '安装驱动失败'))
  }
}

async function handleRemoveMouseFilter() {
  try {
    await hotkeyStore.removeMouseFilter()
    message.success('鼠标过滤器已移除，不影响按键功能')
  } catch (error: unknown) {
    console.error('移除鼠标过滤器失败:', error)
    message.error(errorText(error, '移除鼠标过滤器失败'))
  }
}

async function handleUninstallDriver() {
  try {
    await hotkeyStore.uninstallDriver()
    message.success('驱动已卸载，重启电脑后彻底生效')
  } catch (error: unknown) {
    console.error('卸载按键驱动失败:', error)
    message.error(errorText(error, '卸载驱动失败'))
  }
}

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
  // 原生 input 替代 n-input 后需自行防护：IME 组合中不录入；Esc 取消录入
  if (e.isComposing)
    return
  if (e.key === 'Escape') {
    ;(e.target as HTMLInputElement)?.blur()
    return
  }
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
  // 原生 input 替代 n-input 后需自行防护：IME 组合中不录入；Esc 取消录入
  if (e.isComposing)
    return
  if (e.key === 'Escape') {
    ;(e.target as HTMLInputElement)?.blur()
    return
  }
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
  // 原生 input 替代 n-input 后需自行防护：IME 组合中不录入；Esc 取消录入
  if (e.isComposing)
    return
  if (e.key === 'Escape') {
    ;(e.target as HTMLInputElement)?.blur()
    return
  }
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
    message.error(errorText(error, '保存失败，请检查输入'))
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
  <div class="h-full px-5 py-3.5">
    <PageHeader title="按键">
      <template #extra>
        <div class="status-badge" :class="statusClass">
          <span class="dot" />
          <span>{{ statusText }}</span>
          <span class="hint">{{ formValue.startHotkey || '—' }} 开始 · {{ formValue.stopHotkey || '—' }} 停止</span>
        </div>
      </template>
    </PageHeader>

    <n-alert
      v-if="showDriverInstall"
      type="warning"
      title="按键驱动未安装"
      class="mx-auto mb-3 max-w-[480px]"
    >
      <p>
        剑网三的反作弊会拦截普通模拟按键，本功能需要 Interception <b>键盘</b>驱动（内核级）。
        安装只保留键盘过滤器，<b>不会触碰鼠标</b>；安装后需重启电脑一次。
      </p>
      <n-popconfirm
        :positive-button-props="{ loading: driverBusy }"
        @positive-click="handleInstallDriver"
      >
        <template #trigger>
          <n-button size="small" type="primary" class="mt-2" :loading="driverBusy">
            安装按键驱动
          </n-button>
        </template>
        将安装系统内核驱动（仅键盘过滤器），需重启电脑后生效。确认安装？
      </n-popconfirm>
    </n-alert>

    <n-alert
      v-else-if="showDriverReboot"
      type="info"
      title="驱动已安装，等待重启"
      class="mx-auto mb-3 max-w-[480px]"
    >
      按键驱动已安装但尚未加载，请<b>重启电脑</b>后再使用按键功能。
    </n-alert>

    <n-alert
      v-if="showMouseFilterWarn"
      type="warning"
      title="检测到多余的鼠标过滤器"
      class="mx-auto mb-3 max-w-[480px]"
    >
      <p>
        系统中残留 Interception <b>鼠标</b>过滤器（旧版安装包遗留），可能在重启后导致鼠标失灵。
        本功能只需要键盘过滤器，建议立即移除，不影响按键功能。
      </p>
      <n-button
        size="small"
        type="warning"
        class="mt-2"
        :loading="driverBusy"
        @click="handleRemoveMouseFilter"
      >
        移除鼠标过滤器
      </n-button>
    </n-alert>

    <n-alert v-if="status.lastError" type="error" title="错误" class="mx-auto mb-3 max-w-[480px]">
      {{ status.lastError }}
    </n-alert>

    <n-spin :show="loading">
      <div v-if="config" class="mx-auto max-w-[480px]">
        <div class="paper-card px-4.5 py-4">
          <div class="section-label">
            按键行为
          </div>

          <div class="form-row">
            <span>按键模式</span>
            <div class="flex items-center gap-2">
              <n-radio-group
                :value="formValue.keyMode"
                size="small"
                :disabled="!isWindows"
                @update:value="(val: string) => formValue.keyMode = val as 'global' | 'window'"
              >
                <n-radio-button value="global">
                  全局
                </n-radio-button>
                <n-radio-button value="window">
                  窗口
                </n-radio-button>
              </n-radio-group>
              <n-popover trigger="hover" placement="top" style="max-width: 280px">
                <template #trigger>
                  <span class="info-icon">ⓘ</span>
                </template>
                <p>窗口模式：按键发送到指定窗口，窗口不在前台也能接收。</p>
                <p>部分游戏或应用可能屏蔽此方式的按键。</p>
                <p>目标窗口关闭后任务会自动停止。仅支持 Windows。</p>
              </n-popover>
              <n-text v-if="!isWindows" depth="3" class="text-xs">
                (仅 Windows)
              </n-text>
            </div>
          </div>

          <div v-if="isWindowMode && isWindows" class="form-row !items-start">
            <span class="pt-1">目标窗口</span>
            <div class="ml-4 flex-1">
              <n-input-group>
                <n-input
                  v-model:value="windowFilter"
                  size="small"
                  placeholder="关键词过滤…"
                  clearable
                  @update:value="fetchWindows"
                />
                <n-button size="small" :loading="windowLoading" @click="fetchWindows">
                  刷新
                </n-button>
              </n-input-group>
              <n-select
                class="mt-2"
                size="small"
                :value="formValue.targetWindow?.hwnd ?? null"
                :options="windowOptions"
                placeholder="选择目标窗口"
                filterable
                clearable
                :loading="windowLoading"
                @update:value="handleWindowSelect"
              />
            </div>
          </div>

          <div class="form-row">
            <span>触发按键</span>
            <input
              class="keycap-input"
              readonly
              :value="triggerKeyFocused ? '' : formValue.triggerKey"
              :placeholder="triggerKeyFocused ? '请按下按键…' : '点击录入'"
              @focus="triggerKeyFocused = true"
              @blur="triggerKeyFocused = false"
              @keydown="handleTriggerKeyDown"
            >
          </div>

          <div class="form-row">
            <span>触发频率</span>
            <n-input-number
              v-model:value="formValue.intervalMs"
              size="small"
              :min="20"
              :max="60000"
              :step="50"
            >
              <template #suffix>
                毫秒/次
              </template>
            </n-input-number>
          </div>

          <div class="card-divider" />
          <div class="section-label">
            控制热键 <span class="font-normal">（软件后台时也生效）</span>
          </div>

          <div class="form-row">
            <span>开始热键</span>
            <input
              class="keycap-input"
              readonly
              :value="startHotkeyFocused ? '' : formValue.startHotkey"
              :placeholder="startHotkeyFocused ? '请按下按键…' : '点击录入（支持组合键）'"
              @focus="startHotkeyFocused = true"
              @blur="startHotkeyFocused = false"
              @keydown="handleStartHotkeyKeyDown"
            >
          </div>

          <div class="form-row">
            <span>结束热键</span>
            <input
              class="keycap-input"
              readonly
              :value="stopHotkeyFocused ? '' : formValue.stopHotkey"
              :placeholder="stopHotkeyFocused ? '请按下按键…' : '点击录入（支持组合键）'"
              @focus="stopHotkeyFocused = true"
              @blur="stopHotkeyFocused = false"
              @keydown="handleStopHotkeyKeyDown"
            >
          </div>

          <div class="mt-4 flex items-center gap-2.5">
            <n-button type="primary" :loading="saving" @click="saveConfig">
              保存配置
            </n-button>
            <n-button :disabled="!status.running" @click="hotkeyStore.stopTask">
              停止任务
            </n-button>
          </div>
        </div>

        <!-- 驱动已装时给出卸载入口；未装时由顶部横幅引导安装 -->
        <div v-if="isWindows && driverState !== 'notInstalled'" class="mt-3 text-center text-[10px]">
          <n-popconfirm
            :positive-button-props="{ loading: driverBusy }"
            @positive-click="handleUninstallDriver"
          >
            <template #trigger>
              <a class="cursor-pointer" style="color: var(--ink-muted)">卸载按键驱动</a>
            </template>
            将卸载 Interception 驱动（重启电脑后彻底生效）。确认卸载？
          </n-popconfirm>
        </div>
      </div>
    </n-spin>
  </div>
</template>

<style scoped>
.section-label {
  font-size: 11px;
  letter-spacing: 1px;
  color: var(--ink-muted);
  margin-bottom: 6px;
}

.form-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
  font-size: 12px;
  color: var(--ink);
}

.card-divider {
  height: 1px;
  background: var(--line-soft);
  margin: 10px 0;
}

.info-icon {
  color: var(--ink-muted);
  font-size: 12px;
  cursor: help;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  border-radius: 6px;
  padding: 6px 12px;
  font-size: 11px;
  border: 1px solid var(--line);
  color: var(--ink-secondary);
  background: var(--paper-card);
}

.status-badge .dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--ink-muted);
}

.status-badge .hint {
  font-size: 10px;
  color: var(--ink-muted);
  border-left: 1px solid var(--line);
  padding-left: 8px;
}

.status-badge.running {
  color: var(--bamboo);
  border-color: var(--bamboo);
  background: var(--bamboo-tint);
}

.status-badge.running .dot {
  background: var(--bamboo);
  animation: breath 1.6s ease-in-out infinite;
}

.status-badge.warn {
  color: var(--ochre);
  border-color: var(--ochre);
  background: var(--ochre-tint);
}

.status-badge.warn .dot {
  background: var(--ochre);
}

@keyframes breath {
  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.35;
  }
}
</style>
