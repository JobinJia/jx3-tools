<script setup lang="ts">
import type { UserSelect } from '@/types'
import { useDialog } from 'shuimo-ui'
import { ref } from 'vue'
import IcRoundSettings from '~icons/ic/round-settings'
import { useKeyboard } from '@/composables/useKeyboard'
import Result from './components/Result.vue'
import SourceTree from './components/SourceTree.vue'

const { basePath, selectDirectory, resetDirectory } = useKeyboard()
const { visible, showDialog } = useDialog()

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
</script>

<template>
  <div class="w-full h-full">
    <div v-if="!basePath" class="w-full h-20 text-center">
      <n-space direction="vertical">
        <p>初次使用时，需要手动设置userdata目录路径</p>
        <m-button @click="selectDirectory">
          选择路径
        </m-button>
      </n-space>
    </div>
    <div v-else class="w-full h-full flex flex-row justify-start">
      <SourceTree type="source" @source="setSource" />
      <m-divider vertical />
      <SourceTree type="target" placeholder="搜索没有键位的账号/角色" @source="setTarget" />
      <m-divider vertical />
      <Result style="margin-left: 20px" :user-select="userSelect" />
    </div>
    <div class="fixed top-2 right-2">
      <n-button shape="circle" @click="showDialog">
        <template #icon>
          <IcRoundSettings />
        </template>
      </n-button>
    </div>
    <m-dialog v-model:visible="visible" class="[&_.m-model-close-btn]:left-[93%]">
      <div>
        <n-space>
          <m-button @click="resetDirectory">
            重置路径
          </m-button>
        </n-space>
      </div>
    </m-dialog>
  </div>
</template>
