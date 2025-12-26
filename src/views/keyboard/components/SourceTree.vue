<script setup lang="ts">
import type { TreeOverrideNodeClickBehavior } from 'naive-ui'
import type { FileEntry } from '@/types'
import { NCheckbox, NInput, NTree, useMessage } from 'naive-ui'
import { h, ref, watch } from 'vue'
import FlatColorIconsFolder from '~icons/flat-color-icons/folder'
import IcRoundFolderOpen from '~icons/ic/round-folder-open'
import IcRoundStar from '~icons/ic/round-star'
import IcRoundStarBorder from '~icons/ic/round-star-border'
import { useKeyboard } from '@/composables/useKeyboard'
import { keyboardService } from '@/services'

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

const { basePath, treeData, loadTree, templates, saveTemplate } = useKeyboard()

const expand = ref(false)
const treeKey = ref(0)
const pattern = ref('')

function expandAll() {
  expand.value = true
  treeKey.value++
}

function collapseAll() {
  expand.value = false
  treeKey.value++
}

// 收藏弹窗相关
const showFavoriteModal = ref(false)
const favoriteNode = ref<FileEntry | null>(null)
const favoriteName = ref('')

function openFavoriteModal(node: FileEntry, e: Event) {
  e.stopPropagation()
  favoriteNode.value = node
  favoriteName.value = ''
  showFavoriteModal.value = true
}

function confirmFavorite() {
  if (!favoriteNode.value)
    return
  const path = findPath(treeData.value, favoriteNode.value.name)
  if (!path)
    return
  // 如果没填名称，默认使用角色名
  const name = favoriteName.value.trim() || favoriteNode.value.name
  saveTemplate(name, '', path, favoriteNode.value.name)
  showFavoriteModal.value = false
  favoriteNode.value = null
}

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

// 检查节点是否已收藏
function isNodeFavorited(node: FileEntry): boolean {
  const path = findPath(treeData.value, node.name)
  return templates.value.some(t => t.sourcePath === path)
}

// 打开文件夹
function handleOpenFolder(node: FileEntry, e: Event) {
  e.stopPropagation()
  const path = findPath(treeData.value, node.name)
  if (path && basePath.value) {
    keyboardService.openFolder(`${basePath.value}/${path}`)
  }
}

function renderLabel(info: { option: FileEntry }) {
  const { option } = info
  const labelText = option?.name || ''

  // 目录节点只显示名称
  if (option?.is_dir) {
    return labelText
  }

  // 非目录节点（角色）显示图标
  const isFav = isNodeFavorited(option)
  const icons = [
    // 打开文件夹图标（所有角色都显示）
    h('span', {
      class: 'folder-icon',
      style: {
        cursor: 'pointer',
        padding: '0 4px',
        marginLeft: '4px',
        display: 'inline-flex',
        alignItems: 'center',
      },
      onClick: (e: Event) => handleOpenFolder(option, e),
    }, [
      h(IcRoundFolderOpen, {
        style: { color: '#666', fontSize: '16px' },
      }),
    ]),
  ]

  // 收藏图标（只对 source 类型显示）
  if (type === 'source') {
    icons.push(
      h('span', {
        class: `favorite-icon ${isFav ? 'is-favorited' : ''}`,
        style: {
          cursor: 'pointer',
          padding: '0 4px',
          display: 'inline-flex',
          alignItems: 'center',
        },
        onClick: (e: Event) => {
          e.stopPropagation()
          if (isFav) {
            message.info('已在常用键位中')
          } else {
            openFavoriteModal(option, e)
          }
        },
      }, [
        h(isFav ? IcRoundStar : IcRoundStarBorder, {
          style: { color: isFav ? '#f59e0b' : '#999', fontSize: '16px' },
        }),
      ]),
    )
  }

  return h('span', { class: 'label-with-icons', style: { display: 'inline-flex', alignItems: 'center' } }, [
    h('span', null, labelText),
    ...icons,
  ])
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
</script>

<template>
  <div class="h-screen box-border w-[300px] p-1">
    <div class="flex items-center gap-2 mb-2">
      <n-button size="small" @click="expandAll">
        展开全部
      </n-button>
      <n-button size="small" @click="collapseAll">
        收起全部
      </n-button>
    </div>
    <div class="flex items-center">
      <NInput v-model:value="pattern" :placeholder="placeholder" class="w-full" />
    </div>
    <div class="box-border h-[calc(100vh-20px)] overflow-y-scroll p-b-5">
      <NTree
        :key="treeKey"
        class="compact-tree"
        :indent="10"
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
        :render-label="renderLabel"
        :on-update:selected-keys="handleSelectedKeys"
        :default-expand-all="expand"
      />
    </div>

    <!-- 收藏弹窗 -->
    <m-dialog v-model:visible="showFavoriteModal" title="保存为常用键位">
      <div class="flex flex-col h-full">
        <div class="flex-1 p-4">
          <m-input v-model="favoriteName" placeholder="保存名称(如：万灵)" />
        </div>
        <div class="flex justify-end gap-2 p-4 border-t border-gray-200">
          <m-button @click="showFavoriteModal = false">
            取消
          </m-button>
          <m-button type="primary" @click="confirmFavorite">
            保存
          </m-button>
        </div>
      </div>
    </m-dialog>
  </div>
</template>

<style scoped>
:global(.m-model-close-btn) {
  left: 93% !important;
}

.compact-tree :deep(.n-tree-node) {
  padding: 2px 0;
}

.compact-tree :deep(.n-tree-node-content) {
  padding: 2px 4px;
}

/* hover 时显示图标 */
.compact-tree :deep(.label-with-icons .folder-icon),
.compact-tree :deep(.label-with-icons .favorite-icon) {
  opacity: 0;
  transition: opacity 0.2s;
}

.compact-tree :deep(.n-tree-node:hover .label-with-icons .folder-icon),
.compact-tree :deep(.n-tree-node:hover .label-with-icons .favorite-icon),
.compact-tree :deep(.label-with-icons .favorite-icon.is-favorited) {
  opacity: 1;
}
</style>
