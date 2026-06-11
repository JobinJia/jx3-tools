<script setup lang="ts">
import type { KeyboardTemplate, UserSelect } from '@/types'
import { ref, watch } from 'vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import { useKeyboard } from '@/composables/useKeyboard'
import CloudSync from './components/CloudSync.vue'
import Result from './components/Result.vue'
import SourceTree from './components/SourceTree.vue'
import TemplateList from './components/TemplateList.vue'

const { basePath, changeDirectory, templates, loadTree } = useKeyboard()

const sourceTab = ref<'all' | 'favorites'>('all')

const cloudVisible = ref(false)

// 源/目标独立搜索：两边找的本来就是不同角色，全局一个搜索框会互相干扰
const sourcePattern = ref('')
const targetPattern = ref('')

// 「全部」树内只显示已收藏的角色（快速定位常用键位）
const favOnly = ref(false)

// 页面级统一控制两棵树的展开/收起（树组件自身不再带按钮）
const sourceTreeRef = ref<InstanceType<typeof SourceTree> | null>(null)
const targetTreeRef = ref<InstanceType<typeof SourceTree> | null>(null)
const allExpanded = ref(false)

function toggleExpandAll() {
  allExpanded.value = !allExpanded.value
  if (allExpanded.value) {
    sourceTreeRef.value?.expandAll()
    targetTreeRef.value?.expandAll()
  } else {
    sourceTreeRef.value?.collapseAll()
    targetTreeRef.value?.collapseAll()
  }
}

const userSelect = ref<UserSelect>({
  source: '',
  sourcePath: '',
  target: '',
  targetPath: '',
})

// 切换 userdata 目录后旧的源/目标选择已无意义，清空避免误操作
watch(basePath, () => {
  userSelect.value = { source: '', sourcePath: '', target: '', targetPath: '' }
})

function setSource(val: { name: string, path: string }) {
  userSelect.value.source = val.name
  userSelect.value.sourcePath = val.path
}

function setTarget(val: { name: string, path: string }) {
  userSelect.value.target = val.name
  userSelect.value.targetPath = val.path
}

function selectTemplate(template: KeyboardTemplate) {
  userSelect.value.source = template.name
  userSelect.value.sourcePath = template.sourcePath
}
</script>

<template>
  <div class="h-full flex flex-col px-5 py-3.5">
    <!-- 空状态：未选择 userdata 目录 -->
    <div v-if="!basePath" class="flex flex-1 items-center justify-center">
      <div class="paper-card max-w-[360px] p-8 text-center">
        <div class="empty-seal">
          键
        </div>
        <div class="mt-3 text-sm" style="color: var(--ink)">
          初次使用需要选择游戏 userdata 目录
        </div>
        <div class="mt-1 text-xs" style="color: var(--ink-muted)">
          通常位于 …/SeasunGame/Game/JX3/bin/userdata
        </div>
        <n-button type="primary" class="mt-4" @click="changeDirectory">
          选择 userdata 目录
        </n-button>
      </div>
    </div>

    <template v-else>
      <PageHeader title="改键">
        <template #extra>
          <n-button size="tiny" quaternary class="mr-2" @click="cloudVisible = true">
            ☁ 云同步
          </n-button>
          <n-button size="tiny" quaternary class="mr-2" @click="toggleExpandAll">
            {{ allExpanded ? '收起全部' : '展开全部' }}
          </n-button>
          <div class="path-capsule">
            <span style="color: var(--ink-muted)">📁</span>
            <n-tooltip trigger="hover">
              <template #trigger>
                <span class="path-text">{{ basePath }}</span>
              </template>
              {{ basePath }}
            </n-tooltip>
            <a @click="changeDirectory">切换</a>
            <n-tooltip trigger="hover">
              <template #trigger>
                <a @click="loadTree">⟳</a>
              </template>
              刷新所有键位数据
            </n-tooltip>
          </div>
        </template>
      </PageHeader>

      <div class="min-h-0 flex flex-1 gap-3.5">
        <!-- 源卡片 -->
        <div class="paper-card min-w-0 flex flex-1 flex-col overflow-hidden">
          <div class="card-head">
            <span class="flex-shrink-0 text-xs font-600" style="color: var(--ink)">
              <span class="head-dot" style="background: var(--cinnabar)" />源角色
            </span>
            <n-radio-group v-model:value="sourceTab" size="small" class="flex-shrink-0">
              <n-radio-button value="all">
                全部
              </n-radio-button>
              <n-radio-button value="favorites">
                常用{{ templates.length > 0 ? ` (${templates.length})` : '' }}
              </n-radio-button>
            </n-radio-group>
            <template v-if="sourceTab === 'all'">
              <button
                class="fav-filter"
                :class="{ active: favOnly }"
                :disabled="templates.length === 0"
                :title="templates.length === 0 ? '还没有收藏过常用键位' : (favOnly ? '显示全部角色' : '只显示已收藏的角色')"
                @click="favOnly = !favOnly"
              >
                {{ favOnly ? '★' : '☆' }}
              </button>
              <n-input
                v-model:value="sourcePattern"
                size="tiny"
                clearable
                placeholder="搜索源角色…"
                class="ml-auto min-w-[80px] flex-1"
                style="max-width: 170px"
              />
            </template>
          </div>
          <div class="flex-1 overflow-hidden">
            <SourceTree
              v-show="sourceTab === 'all'"
              ref="sourceTreeRef"
              type="source"
              :pattern="sourcePattern"
              :fav-only="favOnly"
              @source="setSource"
            />
            <TemplateList
              v-show="sourceTab === 'favorites'"
              :selected-path="userSelect.sourcePath"
              @select="selectTemplate"
            />
          </div>
        </div>

        <!-- 流向箭头 -->
        <div class="w-6 flex flex-shrink-0 items-center justify-center text-base" style="color: var(--cinnabar)">
          ➜
        </div>

        <!-- 目标卡片 -->
        <div class="paper-card min-w-0 flex flex-1 flex-col overflow-hidden">
          <div class="card-head">
            <span class="flex-shrink-0 text-xs font-600" style="color: var(--ink)">
              <span class="head-dot" style="background: var(--indigo)" />目标角色
            </span>
            <n-input
              v-model:value="targetPattern"
              size="tiny"
              clearable
              placeholder="搜索目标角色…"
              class="ml-auto min-w-[80px] flex-1"
              style="max-width: 170px"
            />
          </div>
          <div class="flex-1 overflow-hidden">
            <SourceTree
              ref="targetTreeRef"
              type="target"
              :pattern="targetPattern"
              @source="setTarget"
            />
          </div>
        </div>
      </div>

      <!-- 底部操作条：选择摘要 + 复制 -->
      <Result class="mt-3" :user-select="userSelect" />

      <CloudSync v-model:show="cloudVisible" :user-select="userSelect" />
    </template>
  </div>
</template>

<style scoped>
.card-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border-bottom: 1px solid var(--line-soft);
}

.head-dot {
  display: inline-block;
  width: 7px;
  height: 7px;
  border-radius: 1px;
  margin-right: 6px;
  vertical-align: middle;
}

.fav-filter {
  width: 22px;
  height: 22px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  line-height: 1;
  white-space: nowrap;
  color: var(--ink-muted);
  border: 1px solid var(--line);
  border-radius: 50%;
  background: transparent;
  cursor: pointer;
  transition:
    color 0.2s,
    border-color 0.2s,
    background 0.2s;
}

.fav-filter:hover:not(:disabled) {
  color: var(--ochre);
  border-color: var(--ochre);
}

.fav-filter.active {
  color: var(--ochre);
  border-color: var(--ochre);
  background: var(--ochre-tint);
}

.fav-filter:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.path-capsule {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--paper-card);
  border: 1px solid var(--line);
  border-radius: 6px;
  padding: 5px 10px;
  font-size: 11px;
  color: var(--ink-secondary);
}

.path-capsule .path-text {
  max-width: 240px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  direction: rtl;
  unicode-bidi: plaintext;
}

.path-capsule a {
  color: var(--cinnabar);
  cursor: pointer;
}

.empty-seal {
  width: 56px;
  height: 56px;
  margin: 0 auto;
  background: var(--cinnabar);
  border-radius: 8px;
  color: #f5efe2;
  font-size: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Songti SC', 'STSong', 'SimSun', serif;
}
</style>
