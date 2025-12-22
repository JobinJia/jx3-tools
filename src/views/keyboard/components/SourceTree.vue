<script setup lang="ts">
import type { TreeOverrideNodeClickBehavior } from 'naive-ui'
import type { FileEntry } from '@/types'
import { NCheckbox, NInput, NTree, useMessage } from 'naive-ui'
import { computed, h, ref, watch } from 'vue'
import FlatColorIconsFolder from '~icons/flat-color-icons/folder'
import { useKeyboard } from '@/composables/useKeyboard'

const {
  type = 'source',
  placeholder = '搜索带键位的账号/角色名称',
} = defineProps<{
  type?: 'source' | 'target'
  placeholder?: string
}>()

const emit = defineEmits<{
  source: [source: { name: string, path: string }]
}>()

const message = useMessage()
const source = ref('')

const { basePath, treeData, loadTree } = useKeyboard()

const expand = ref(false)
const pattern = ref('')

// Load directory tree when basePath changes
watch(() => basePath.value, async (newPath) => {
  if (newPath) {
    await loadTree()
  }
}, { immediate: true })

function handleFilter(ptn: string, node: FileEntry) {
  return node.name.includes(ptn)
}

const override: TreeOverrideNodeClickBehavior = ({ option }) => {
  if (option.children) {
    return 'toggleExpand'
  }
  return 'default'
}

function renderPrefix(info: { option: FileEntry, checked: boolean, selected: boolean }) {
  const { option, selected } = info
  if (option?.children) {
    return h(FlatColorIconsFolder)
  }
  return h(NCheckbox, {
    'checked': selected,
    'onUpdate:checked': (value: boolean) => {
      info.selected = value
    },
  })
}

function handleSelectedKeys(
  _keys: Array<string | number>,
  _option: Array<FileEntry | null>,
  meta: { node: FileEntry | null, action: 'select' | 'unselect' },
) {
  const { node, action } = meta
  if (node?.children) {
    return
  }
  if (action === 'select') {
    message.success(`选择${node?.name}`)
    source.value = node!.name
  } else {
    message.success(`取消选择${node?.name}`)
  }
}

// Find the path to a node by name
function findPath(data: FileEntry[], targetName: string): string | null {
  function helper(entries: FileEntry[], currentPath: string): string | null {
    for (const entry of entries) {
      const newPath = `${currentPath}/${entry.name}`
      if (entry.name === targetName) {
        return newPath
      }
      // Search children regardless of is_dir flag (some leaf nodes may have children)
      if (entry.children && entry.children.length > 0) {
        const result = helper(entry.children, newPath)
        if (result) {
          return result
        }
      }
    }
    return null
  }

  const result = helper(data, '')
  return result ? result.substring(1) : null
}

watch(source, (val) => {
  const path = findPath(treeData.value, val)
  emit('source', {
    name: val,
    path: path ?? '',
  })
})

watch(pattern, () => {
  if (!expand.value) {
    expand.value = true
  }
})

function handleRefresh() {
  loadTree()
}

const clazz = computed(() => {
  return type === 'source' ? 'w-[70%]' : 'w-[60%]'
})
</script>

<template>
  <div class="h-screen box-border w-[300px] p-1">
    <div class="flex items-center justify-between">
      <NInput v-model:value="pattern" :class="clazz" :placeholder="placeholder" />
      <n-button type="default" @click="expand = !expand">
        {{ expand ? '收起' : '展开' }}
      </n-button>
      <n-tooltip v-if="type === 'target'" trigger="hover">
        <template #trigger>
          <n-button @click="handleRefresh">
            刷新
          </n-button>
        </template>
        如果没有找到角色，猛猛刷新
      </n-tooltip>
    </div>
    <div class="box-border h-[calc(100vh-20px)] overflow-y-scroll p-b-5">
      <NTree
        :pattern="pattern"
        show-line
        :override-default-node-click-behavior="override"
        :data="treeData"
        block-line
        key-field="id"
        label-field="name"
        :filter="handleFilter"
        :show-irrelevant-nodes="false"
        expand-on-click
        :render-prefix="renderPrefix"
        :on-update:selected-keys="handleSelectedKeys"
        :default-expand-all="expand"
      />
    </div>
  </div>
</template>
