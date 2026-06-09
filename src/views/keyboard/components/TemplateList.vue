<script setup lang="ts">
import type { KeyboardTemplate } from '@/types'
import { computed } from 'vue'
import IcRoundDelete from '~icons/ic/round-delete'
import { useKeyboard } from '@/composables/useKeyboard'

const { selectedPath } = defineProps<{
  selectedPath?: string
}>()

const emit = defineEmits<{
  select: [template: KeyboardTemplate]
}>()

const { templates, deleteTemplate } = useKeyboard()

const isEmpty = computed(() => templates.value.length === 0)

function handleSelect(template: KeyboardTemplate) {
  emit('select', template)
}

function formatTime(timestamp: number) {
  const date = new Date(timestamp)
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')
  const seconds = String(date.getSeconds()).padStart(2, '0')
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`
}
</script>

<template>
  <div class="h-full flex flex-col p-2">
    <n-empty v-if="isEmpty" description="暂无常用键位" class="mt-10">
      <template #extra>
        <n-text depth="3" class="text-xs">
          在"所有键位"中选择角色后可保存为常用
        </n-text>
      </template>
    </n-empty>

    <div v-else class="min-h-0 flex-1 overflow-y-auto">
      <div
        v-for="template in templates"
        :key="template.id"
        class="mb-2 flex cursor-pointer items-center justify-between border rounded p-3 transition-colors"
        :class="selectedPath === template.sourcePath ? 'tpl-item tpl-selected' : 'tpl-item'"
        @click="handleSelect(template)"
      >
        <div class="min-w-0 flex flex-1 flex-col">
          <n-text strong class="truncate">
            {{ template.name }}
          </n-text>
          <n-text depth="3" class="truncate text-xs">
            {{ template.characterName }}
          </n-text>
          <n-text depth="3" class="mt-1 truncate text-xs">
            收藏于 {{ formatTime(template.createdAt) }}
          </n-text>
        </div>
        <n-popconfirm positive-text="确定" negative-text="取消" @positive-click.stop="deleteTemplate(template.id)">
          <template #trigger>
            <n-button size="tiny" quaternary circle @click.stop>
              <template #icon>
                <IcRoundDelete />
              </template>
            </n-button>
          </template>
          确定删除？
        </n-popconfirm>
      </div>
    </div>
  </div>
</template>

<style scoped>
.tpl-item {
  background: var(--paper-card);
  border-color: var(--line);
}

.tpl-item:hover {
  border-color: var(--ink-muted);
}

.tpl-selected {
  background: var(--bamboo-tint);
  border-color: var(--bamboo);
}
</style>
