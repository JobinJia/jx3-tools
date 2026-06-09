<script setup lang="ts">
import type { UserSelect } from '@/types'
import { ref } from 'vue'
import { useKeyboard } from '@/composables/useKeyboard'
import { useRecentOps } from '@/composables/useRecentOps'

const props = defineProps<{
  userSelect: UserSelect
}>()

const { copyKeyboardConfig, copyLoading } = useKeyboard()
const { recentOps, addRecentOp, formatOpTime } = useRecentOps()

const faqExpanded = ref(false)

async function handleCopy() {
  const ok = await copyKeyboardConfig(props.userSelect)
  if (ok)
    addRecentOp(props.userSelect.source, props.userSelect.target)
}
</script>

<template>
  <div class="flex flex-col gap-3 h-full min-w-[170px] flex-1">
    <div class="paper-card p-3.5">
      <div class="text-[11px] mb-2" style="color: var(--ink-muted)">
        本次操作
      </div>
      <div class="text-xs leading-7" style="color: var(--ink)">
        <div>
          <span class="mr-2" style="color: var(--cinnabar)">源</span>{{ userSelect.source || '未选择' }}
        </div>
        <div class="pl-3.5" style="color: var(--ink-muted)">
          ↓ 覆盖
        </div>
        <div>
          <span class="mr-2" style="color: var(--indigo)">目</span>{{ userSelect.target || '未选择' }}
        </div>
      </div>
      <button
        class="copy-btn"
        :disabled="!(userSelect.source && userSelect.target) || copyLoading"
        @click="handleCopy"
      >
        {{ copyLoading ? '复制中…' : '复 制 键 位' }}
      </button>
      <div class="text-[9px] text-center mt-1.5" style="color: var(--ink-muted)">
        目标角色现有键位将被覆盖，不可撤销
      </div>
    </div>

    <div class="paper-card p-3 flex-1 overflow-y-auto min-h-0">
      <div class="text-[11px] mb-1.5" style="color: var(--ink-muted)">
        最近操作
      </div>
      <div v-if="recentOps.length === 0" class="text-[10px]" style="color: var(--ink-muted)">
        暂无记录
      </div>
      <div
        v-for="op in recentOps"
        :key="op.at"
        class="text-[10px] leading-7 truncate"
        style="color: var(--ink-secondary)"
      >
        <span style="color: var(--bamboo)">✓</span> {{ op.source }} → {{ op.target }}
        <span class="float-right" style="color: var(--ink-muted)">{{ formatOpTime(op.at) }}</span>
      </div>
    </div>

    <div class="text-[10px]" style="color: var(--ink-muted)">
      <a class="cursor-pointer" style="color: var(--indigo)" @click="faqExpanded = !faqExpanded">
        常见问题 {{ faqExpanded ? '▴' : '▾' }}
      </a>
      <div v-if="faqExpanded" class="mt-1.5 leading-relaxed">
        <p><b>自己带键位的账号</b>需要在游戏里<b>关闭同步到服务器</b>，这样键位才能在本地得到保存。</p>
        <p class="mt-1">
          <b>新账号</b>登入到角色选择界面后选中角色（不进入游戏），点击「⟳ 刷新」即可搜索到该角色。
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.copy-btn {
  width: 100%;
  margin-top: 12px;
  padding: 8px 0;
  border: none;
  border-radius: 6px;
  background: var(--cinnabar);
  color: #f7f2e4;
  font-size: 13px;
  letter-spacing: 4px;
  font-family: 'Songti SC', 'STSong', 'SimSun', serif;
  cursor: pointer;
  transition: background 0.2s;
}

.copy-btn:hover:not(:disabled) {
  background: var(--cinnabar-hover);
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
