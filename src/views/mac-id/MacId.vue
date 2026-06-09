<script setup lang="ts">
import { onMounted, ref } from 'vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import { useMac } from '@/composables/useMac'

const {
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

const helpExpanded = ref(false)

onMounted(() => {
  fetchMacAddress()
})
</script>

<template>
  <div class="p-5 h-full">
    <PageHeader title="MAC地址" description="查看、随机修改与还原网卡物理地址" />

    <n-alert v-if="error" type="error" :title="error" class="max-w-[430px] mx-auto mb-3" />

    <n-spin :show="loading">
      <div class="max-w-[430px] mx-auto">
        <div class="paper-card p-5 text-center">
          <div class="text-[10px] tracking-[2px]" style="color: var(--ink-muted)">
            当前地址
          </div>
          <div class="text-mono text-[26px] tracking-[3px] mt-2" style="color: var(--ink)">
            {{ currentAddress || '——' }}
          </div>
          <div class="mt-2.5">
            <n-tag :type="isChanged ? 'warning' : 'success'" size="small">
              {{ isChanged ? '已修改' : '原始地址' }}
            </n-tag>
          </div>
          <div class="text-[10px] mt-3" style="color: var(--ink-muted)">
            原始地址 <span class="text-mono tracking-wider ml-1">{{ originalAddress || '——' }}</span>
          </div>
          <div class="flex gap-2.5 justify-center mt-4.5">
            <n-button type="primary" :loading="changing" @click="changeMacAddress">
              随机修改
            </n-button>
            <n-button :loading="restoring" @click="restoreMacAddress">
              还原地址
            </n-button>
          </div>
        </div>

        <div class="paper-card px-4 py-3 mt-3 flex items-center justify-between">
          <div>
            <div class="text-xs" style="color: var(--ink)">
              重启自动还原
            </div>
            <div class="text-[10px] mt-0.5" style="color: var(--ink-muted)">
              开机时自动恢复原始地址（计划任务）
            </div>
          </div>
          <n-switch v-model:value="autoRestoreEnabled" @update:value="setAutoRestore" />
        </div>

        <div class="text-center mt-3 text-[10px]" style="color: var(--ink-muted)">
          什么是 MAC 地址？为什么要修改？
          <a class="cursor-pointer" style="color: var(--indigo)" @click="helpExpanded = !helpExpanded">
            {{ helpExpanded ? '收起说明 ▴' : '展开说明 ▾' }}
          </a>
        </div>
        <div
          v-if="helpExpanded"
          class="paper-card p-4 mt-2 text-xs leading-relaxed"
          style="color: var(--ink-secondary)"
        >
          <p><b>什么是MAC地址？</b></p>
          <p>MAC地址是网络设备的唯一标识符，由48位二进制数字组成，通常表示为12个十六进制数字。</p>
          <p class="mt-2">
            <b>为什么要修改MAC地址？</b>
          </p>
          <p>修改MAC地址可用于增强隐私保护、绕过基于MAC地址的网络访问控制等。请确保您的操作符合相关法律法规。</p>
        </div>
      </div>
    </n-spin>
  </div>
</template>
