<script setup lang="ts">
import type { UserSelect } from '@/types'
import { ref, watchPostEffect } from 'vue'
import { useKeyboard } from '@/composables/useKeyboard'

const props = defineProps<{
  userSelect: UserSelect
}>()

const formValue = ref({
  source: '',
  target: '',
})

watchPostEffect(() => {
  formValue.value.target = props.userSelect.target
  formValue.value.source = props.userSelect.source
})

const { copyKeyboardConfig } = useKeyboard()

function gogogo() {
  copyKeyboardConfig(props.userSelect)
}
</script>

<template>
  <div class="h-full flex-1 relative box-border">
    <n-form
      style="margin-top: 20px"
      :label-width="100"
      :model="formValue"
      size="small"
      label-placement="left"
      label-align="left"
    >
      <n-form-item class="w-80%" label="带键位的角色" path="user.name">
        <n-input v-model:value="formValue.source" disabled placeholder="请从左边勾选" />
      </n-form-item>
      <n-form-item class="w-80%" label="没键位的角色" path="user.age">
        <n-input v-model:value="formValue.target" disabled placeholder="请从左边勾选" />
      </n-form-item>
      <n-form-item>
        <n-button type="primary" :disabled="!(formValue.target && formValue.source)" @click="gogogo">
          替换键位
        </n-button>
      </n-form-item>
    </n-form>
    <n-alert v-if="formValue.target && formValue.source" class="w-80%" title="提示" type="info" :bordered="false">
      角色 <b>{{ formValue.target }}</b> 使用 <b> {{ formValue.source }} </b> 的键位
    </n-alert>
    <n-alert class="w-80% mt-2" :show-icon="false" title="常见问题及方案">
      <p><b>自己带键位的账号</b>需要在游戏里<b>关闭同步到服务器</b>，这样键位在才能本地得到保存</p>
      <p>如果是<b>新账号，登入到游戏角色选择界面后，选中需要改键位的角色，别进入游戏，停在这个游戏界面。</b>然后点击刷新就能搜索到这个角色了</p>
      <p>初次始用需要手动指定剑三的userdata所在的目录路径。之后就不用再设置。</p>
    </n-alert>
  </div>
</template>
