<script setup lang="ts">
import type { KeyboardTemplate, UserSelect } from '@/types'
import { ref } from 'vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import { useKeyboard } from '@/composables/useKeyboard'
import Result from './components/Result.vue'
import SourceTree from './components/SourceTree.vue'
import TemplateList from './components/TemplateList.vue'

const { basePath, changeDirectory, templates, loadTree } = useKeyboard()

const sourceTab = ref<'all' | 'favorites'>('all')

const userSelect = ref<UserSelect>({
  source: '',
  sourcePath: '',
  target: '',
  targetPath: '',
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
  <div class="p-5 h-full flex flex-col">
    <!-- 空状态：未选择 userdata 目录 -->
    <div v-if="!basePath" class="flex-1 flex items-center justify-center">
      <div class="paper-card p-8 text-center max-w-[360px]">
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
      <PageHeader title="改键" description="在账号与角色之间复制键位配置">
        <template #extra>
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

      <div class="flex-1 flex gap-3.5 min-h-0">
        <!-- 源卡片 -->
        <div class="paper-card w-[270px] flex flex-col overflow-hidden flex-shrink-0">
          <div class="card-head">
            <span class="text-xs font-600" style="color: var(--ink)">源角色</span>
            <n-radio-group v-model:value="sourceTab" size="small">
              <n-radio-button value="all">
                全部
              </n-radio-button>
              <n-radio-button value="favorites">
                常用 {{ templates.length }}
              </n-radio-button>
            </n-radio-group>
          </div>
          <div class="flex-1 overflow-hidden">
            <SourceTree v-show="sourceTab === 'all'" type="source" @source="setSource" />
            <TemplateList
              v-show="sourceTab === 'favorites'"
              :selected-path="userSelect.sourcePath"
              @select="selectTemplate"
            />
          </div>
        </div>

        <!-- 流向箭头 -->
        <div class="flex items-center justify-center w-6 flex-shrink-0 text-base" style="color: var(--cinnabar)">
          ➜
        </div>

        <!-- 目标卡片 -->
        <div class="paper-card w-[270px] flex flex-col overflow-hidden flex-shrink-0">
          <div class="card-head">
            <span class="text-xs font-600" style="color: var(--ink)">目标角色</span>
            <span class="text-[10px]" style="color: var(--ink-muted)">键位将被覆盖</span>
          </div>
          <div class="flex-1 overflow-hidden">
            <SourceTree type="target" placeholder="搜索没有键位的账号/角色" @source="setTarget" />
          </div>
        </div>

        <!-- 操作面板 -->
        <Result :user-select="userSelect" />
      </div>
    </template>
  </div>
</template>

<style scoped>
.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-bottom: 1px solid var(--line-soft);
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
