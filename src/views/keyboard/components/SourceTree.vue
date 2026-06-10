<script setup lang="ts">
import type { TreeOption, TreeOverrideNodeClickBehavior } from 'naive-ui'
import type { FileEntry } from '@/types'
import { NCheckbox, NTree, useMessage } from 'naive-ui'
import { computed, h, ref, watch } from 'vue'
import FlatColorIconsFolder from '~icons/flat-color-icons/folder'
import IcRoundFolderOpen from '~icons/ic/round-folder-open'
import IcRoundStar from '~icons/ic/round-star'
import IcRoundStarBorder from '~icons/ic/round-star-border'
import { useKeyboard } from '@/composables/useKeyboard'
import { keyboardService } from '@/services'

const {
  type = 'source',
  pattern = '',
  favOnly = false,
} = defineProps<{
  type?: 'source' | 'target'
  /** 搜索词由页面级全局搜索框下发（同时过滤两棵树） */
  pattern?: string
  /** 只显示已收藏（常用键位）的角色，可与搜索词叠加 */
  favOnly?: boolean
}>()

const emit = defineEmits<{
  source: [source: { name: string, path: string }]
}>()

const message = useMessage()

const { basePath, treeData, loadTree, templates, saveTemplate } = useKeyboard()

const expand = ref(false)
const treeKey = ref(0)

function expandAll() {
  expand.value = true
  treeKey.value++
}

function collapseAll() {
  expand.value = false
  treeKey.value++
}

// 展开/收起由页面级工具行统一控制（保持树组件纯粹）
defineExpose({ expandAll, collapseAll })

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
  const path = findPathById(treeData.value, favoriteNode.value.id)
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

// NTree 的 data 实际是 FileEntry[]（key-field="id"、label-field="name"），
// 回调签名按 naive-ui 的 TreeOption 声明，这里收窄回真实类型
function toFileEntry(option: TreeOption): FileEntry {
  return option as unknown as FileEntry
}

function handleFilter(_ptn: string, node: TreeOption) {
  const entry = toFileEntry(node)
  const matchesText = !pattern || entry.name.includes(pattern)
  // 只看收藏：仅角色节点（is_dir=false）且已收藏，可与搜索词叠加
  if (favOnly)
    return !entry.is_dir && isNodeFavorited(entry) && matchesText
  return matchesText
}

// NTree 只在 pattern 非空时启用过滤；favOnly 模式下用不可见哨兵字符撑起过滤
const FAV_SENTINEL = '\u0001'
const effectivePattern = computed(() => (favOnly ? `${FAV_SENTINEL}${pattern}` : pattern))

const override: TreeOverrideNodeClickBehavior = ({ option }) => {
  if (option.children) {
    return 'toggleExpand'
  }
  return 'default'
}

function renderPrefix(info: { option: TreeOption, checked: boolean, selected: boolean }) {
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
  const path = findPathById(treeData.value, node.id)
  return templates.value.some(t => t.sourcePath === path)
}

// 打开文件夹
function handleOpenFolder(node: FileEntry, e: Event) {
  e.stopPropagation()
  const path = findPathById(treeData.value, node.id)
  if (path && basePath.value) {
    keyboardService.openFolder(`${basePath.value}/${path}`)
  }
}

function renderLabel(info: { option: TreeOption }) {
  const option = toFileEntry(info.option)
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
  _option: Array<TreeOption | null>,
  meta: { node: TreeOption | null, action: 'select' | 'unselect' },
) {
  const { action } = meta
  const node = meta.node ? toFileEntry(meta.node) : null
  if (node?.children) {
    return
  }
  if (action === 'select') {
    message.success(`选择${node?.name}`)
    const path = findPathById(treeData.value, node!.id)
    emit('source', { name: node!.name, path: path ?? '' })
  } else {
    message.success(`取消选择${node?.name}`)
  }
}

// 按节点 id 定位路径：同名角色可能出现在不同账号/服务器下，按名字找会错配
function findPathById(data: FileEntry[], targetId: number): string | null {
  function helper(entries: FileEntry[], currentPath: string): string | null {
    for (const entry of entries) {
      const newPath = `${currentPath}/${entry.name}`
      if (entry.id === targetId) {
        return newPath
      }
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

watch(effectivePattern, () => {
  if (!expand.value) {
    expand.value = true
  }
})
</script>

<template>
  <div class="h-full w-full flex flex-col p-2.5">
    <div class="min-h-0 flex-1 overflow-y-auto">
      <NTree
        :key="treeKey"
        class="compact-tree"
        :indent="10"
        :pattern="effectivePattern"
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
      <div class="h-full flex flex-col">
        <div class="flex-1 p-4">
          <m-input v-model="favoriteName" placeholder="保存名称(如：万灵)" />
        </div>
        <div class="flex justify-end gap-2 border-t p-4" style="border-color: var(--line-soft)">
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
