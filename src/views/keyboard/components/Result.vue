<script setup lang="ts">
import type { UserSelect } from '@/types'
import { useKeyboard } from '@/composables/useKeyboard'
import { useRecentOps } from '@/composables/useRecentOps'

const props = defineProps<{
  userSelect: UserSelect
}>()

const { copyKeyboardConfig, copyLoading, syncPluginEnabled } = useKeyboard()
const { recentOps, addRecentOp, formatOpTime } = useRecentOps()

async function handleCopy() {
  const ok = await copyKeyboardConfig(props.userSelect)
  if (ok)
    addRecentOp(props.userSelect.source, props.userSelect.target)
}
</script>

<template>
  <div class="paper-card flex items-center gap-3 px-4 py-2.5">
    <div class="flex items-center gap-2 text-xs">
      <span class="slot-chip slot-source" :class="{ 'slot-empty': !userSelect.source }">
        源 · {{ userSelect.source || '未选择' }}
      </span>
      <span style="color: var(--cinnabar)">➜</span>
      <span class="slot-chip slot-target" :class="{ 'slot-empty': !userSelect.target }">
        目 · {{ userSelect.target || '未选择' }}
      </span>
    </div>

    <div class="ml-auto flex items-center gap-4">
      <n-popover trigger="click" placement="top-end" style="max-width: 340px">
        <template #trigger>
          <a class="bar-link">常见问题</a>
        </template>
        <div class="text-xs leading-relaxed">
          <p><b>自己带键位的账号</b>需要在游戏里<b>关闭同步到服务器</b>，这样键位才能在本地得到保存。</p>
          <p class="mt-1.5">
            <b>新账号</b>登入到角色选择界面后选中角色（不进入游戏），点击「⟳ 刷新」即可搜索到该角色。
          </p>
        </div>
      </n-popover>

      <n-popover trigger="click" placement="top-end" style="width: 300px">
        <template #trigger>
          <a class="bar-link">最近操作</a>
        </template>
        <div class="text-xs">
          <div v-if="recentOps.length === 0" style="color: var(--ink-muted)">
            暂无记录
          </div>
          <div
            v-for="op in recentOps"
            :key="op.at"
            class="truncate leading-6"
          >
            <span style="color: var(--bamboo)">✓</span> {{ op.source }} → {{ op.target }}
            <span class="float-right" style="color: var(--ink-muted)">{{ formatOpTime(op.at) }}</span>
          </div>
        </div>
      </n-popover>

      <n-tooltip trigger="hover" placement="top" style="max-width: 300px">
        <template #trigger>
          <n-checkbox v-model:checked="syncPluginEnabled" size="small">
            <span class="text-xs">同步插件配置</span>
          </n-checkbox>
        </template>
        同时把 interface 下茗伊 / 枫影等插件的角色配置从源角色同步到目标角色（需关闭游戏；目标角色需用插件登录过一次）
      </n-tooltip>

      <button
        class="copy-btn"
        :disabled="!(userSelect.source && userSelect.target) || copyLoading"
        @click="handleCopy"
      >
        {{ copyLoading ? '复制中…' : '复制键位' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.slot-chip {
  border-radius: 5px;
  padding: 4px 10px;
  border: 1px solid;
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.slot-source {
  color: var(--cinnabar);
  border-color: var(--cinnabar);
  background: var(--cinnabar-tint);
}

.slot-target {
  color: var(--indigo);
  border-color: var(--indigo);
  background: var(--indigo-tint);
}

.slot-empty {
  color: var(--ink-muted);
  border-color: var(--line);
  border-style: dashed;
  background: transparent;
}

.bar-link {
  font-size: 11px;
  color: var(--indigo);
  cursor: pointer;
}

.copy-btn {
  padding: 5px 16px;
  border: none;
  border-radius: 5px;
  background: linear-gradient(180deg, var(--cinnabar-hover) 0%, var(--cinnabar) 100%);
  color: #f7f2e4;
  font-size: 12px;
  letter-spacing: 3px;
  /* 字距补偿，让文字整体视觉居中 */
  text-indent: 3px;
  font-family: 'Songti SC', 'STSong', 'SimSun', serif;
  cursor: pointer;
  box-shadow:
    0 1px 3px rgba(156, 47, 35, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.15);
  transition:
    filter 0.15s,
    transform 0.1s,
    box-shadow 0.15s;
}

.copy-btn:hover:not(:disabled) {
  filter: brightness(1.08);
}

.copy-btn:active:not(:disabled) {
  transform: translateY(1px);
  box-shadow:
    0 0 1px rgba(156, 47, 35, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

.copy-btn:focus-visible {
  outline: 2px solid var(--cinnabar);
  outline-offset: 2px;
}

.copy-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
</style>
