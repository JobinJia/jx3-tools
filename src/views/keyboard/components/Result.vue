<script setup lang="ts">
import type { UserSelect } from '@/types'
import { useKeyboard } from '@/composables/useKeyboard'

const props = defineProps<{
  userSelect: UserSelect
}>()

const { copyKeyboardConfig, copyLoading } = useKeyboard()

async function gogogo() {
  await copyKeyboardConfig(props.userSelect)
}
</script>

<template>
  <div class="h-full flex-1 relative box-border pt-4 px-2">
    <!-- 紧凑的源→目标显示 -->
    <div class="flex items-center gap-2 flex-wrap mb-4">
      <n-tag :type="userSelect.source ? 'success' : 'default'" size="medium">
        {{ userSelect.source || '选择源角色' }}
      </n-tag>
      <span class="text-gray-400">→</span>
      <n-tag :type="userSelect.target ? 'warning' : 'default'" size="medium">
        {{ userSelect.target || '选择目标角色' }}
      </n-tag>
      <n-button
        type="primary"
        size="small"
        :loading="copyLoading"
        :disabled="!(userSelect.target && userSelect.source) || copyLoading"
        @click="gogogo"
      >
        替换键位
      </n-button>
    </div>

    <div class="mt-4">
      <n-text depth="2" class="text-sm font-bold">
        常见问题
      </n-text>
      <p class="mt-2 text-sm">
        <b>自己带键位的账号</b>需要在游戏里<b>关闭同步到服务器</b>，这样键位在才能本地得到保存
      </p>
      <p class="mt-2 text-sm">
        <b>新账号</b>登入到游戏角色选择界面后，选中需要改键位的角色，别进入游戏，停在这个界面，然后点击刷新就能搜索到这个角色了
      </p>
    </div>
  </div>
</template>
