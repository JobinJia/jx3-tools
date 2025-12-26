<script setup lang="ts">
import type { KeyboardTemplate, UserSelect } from '@/types'
import { ref } from 'vue'
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
  <div class="w-full h-full">
    <div v-if="!basePath" class="w-full h-20 text-center">
      <n-space direction="vertical">
        <p>初次使用时，需要手动设置userdata目录路径</p>
        <m-button @click="changeDirectory">
          设置键位路径
        </m-button>
      </n-space>
    </div>
    <div v-else class="w-full h-full flex flex-col">
      <div class="flex items-center gap-2 mb-2 px-2">
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-text depth="3" class="max-w-[400px] truncate text-sm">
              {{ basePath }}
            </n-text>
          </template>
          {{ basePath }}
        </n-tooltip>
        <n-button size="tiny" secondary @click="changeDirectory">
          切换路径
        </n-button>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button size="tiny" secondary @click="loadTree">
              刷新
            </n-button>
          </template>
          刷新所有键位数据
        </n-tooltip>
      </div>
      <div class="flex-1 flex flex-row justify-start overflow-hidden">
        <!-- 左侧：源选择（带 tabs） -->
        <div class="flex flex-col flex-shrink-0" style="width: 240px">
          <n-tabs v-model:value="sourceTab" type="line" size="small" class="px-2">
            <n-tab name="all">
              所有键位
            </n-tab>
            <n-tab name="favorites">
              常用键位 ({{ templates.length }})
            </n-tab>
          </n-tabs>
          <div class="flex-1 overflow-hidden">
            <SourceTree v-show="sourceTab === 'all'" type="source" @source="setSource" />
            <TemplateList
              v-show="sourceTab === 'favorites'"
              :selected-path="userSelect.sourcePath"
              @select="selectTemplate"
            />
          </div>
        </div>
        <m-divider vertical />
        <!-- 中间：目标选择 -->
        <SourceTree type="target" placeholder="搜索没有键位的账号/角色" @source="setTarget" />
        <m-divider vertical />
        <!-- 右侧：操作面板 -->
        <Result style="margin-left: 20px" :user-select="userSelect" />
      </div>
    </div>
  </div>
</template>
