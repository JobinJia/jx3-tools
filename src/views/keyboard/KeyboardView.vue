<script setup lang="ts">
import { useDialog } from 'shuimo-ui'
import { ref } from 'vue'
import IcRoundSettings from '~icons/ic/round-settings'
import { useBasePath } from '@/composables/basePath'
import { useKeyboardDir } from '@/composables/keyboardDir'
import Result from './components/Result.vue'
import SourceTree from './components/SourceTree.vue'

const { handleSetDir, handleResetDir } = useKeyboardDir()
const { basePath } = useBasePath()
const { visible, showDialog } = useDialog()

const userSelect = ref({
  source: '',
  sourcePath: '',
  target: '',
  targetPath: '',
})

function setSource(val: any) {
  const { name, path } = val
  userSelect.value.source = name
  userSelect.value.sourcePath = path
}

function setTarget(val: any) {
  const { name, path } = val
  userSelect.value.target = name
  userSelect.value.targetPath = path
}
</script>

<template>
  <div class="w-full h-full">
    <div v-if="!basePath" class="w-full h-20 text-center">
      <n-space direction="vertical">
        <p>初次使用时，需要手动设置userdata目录路径</p>
        <m-button @click="handleSetDir">
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
          <m-button @click="handleResetDir">
            重置路径
          </m-button>
        </n-space>
      </div>
    </m-dialog>
  </div>
</template>
