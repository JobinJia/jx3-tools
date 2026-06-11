<script setup lang="ts">
import { onMounted } from 'vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import { useMac } from '@/composables/useMac'

const {
  adapterName,
  originalAddress,
  currentAddress,
  autoRestoreEnabled,
  loading,
  changing,
  restoring,
  error,
  isChanged,
  fetchMacAddress,
  changeMacAddress,
  restoreMacAddress,
  setAutoRestore,
} = useMac()

onMounted(() => {
  fetchMacAddress()
})
</script>

<template>
  <div class="h-full px-5 py-3.5">
    <PageHeader title="MAC地址" />

    <n-alert v-if="error" type="error" :title="error" class="mx-auto mb-3 max-w-[430px]" />

    <n-spin :show="loading">
      <div class="mx-auto max-w-[430px]">
        <div class="paper-card p-5 text-center">
          <div class="text-[10px] tracking-[2px]" style="color: var(--ink-muted)">
            当前地址<span v-if="adapterName"> · {{ adapterName }}</span>
          </div>
          <div class="text-mono mt-2 text-[26px] tracking-[3px]" style="color: var(--ink)">
            {{ currentAddress || '——' }}
          </div>
          <div class="mt-2.5">
            <n-tag :type="isChanged ? 'warning' : 'success'" size="small">
              {{ isChanged ? '已修改' : '原始地址' }}
            </n-tag>
          </div>
          <div class="mt-3 text-[10px]" style="color: var(--ink-muted)">
            原始地址 <span class="text-mono ml-1 tracking-wider">{{ originalAddress || '——' }}</span>
          </div>
          <div class="mt-4.5 flex justify-center gap-2.5">
            <n-button type="primary" :loading="changing" @click="changeMacAddress">
              随机修改
            </n-button>
            <n-button :loading="restoring" @click="restoreMacAddress">
              还原地址
            </n-button>
          </div>
        </div>

        <div class="paper-card mt-3 flex items-center justify-between px-4 py-3">
          <div class="text-xs" style="color: var(--ink)">
            重启自动还原
          </div>
          <n-switch v-model:value="autoRestoreEnabled" @update:value="setAutoRestore" />
        </div>
      </div>
    </n-spin>
  </div>
</template>
