<script setup lang="ts">
import { onMounted } from 'vue'
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

onMounted(() => {
  fetchMacAddress()
})
</script>

<template>
  <div class="mac-id-container">
    <n-card title="MAC地址管理" class="mac-card">
      <n-spin :show="loading">
        <!-- 简化的表格 -->
        <n-table :bordered="true" :single-line="false">
          <n-tbody>
            <n-tr>
              <n-td>原始MAC地址</n-td>
              <n-td>
                <n-tag type="primary">
                  {{ originalAddress }}
                </n-tag>
              </n-td>
            </n-tr>
            <n-tr>
              <n-td>当前MAC地址</n-td>
              <n-td>
                <n-tag :type="isChanged ? 'warning' : 'success'">
                  {{ currentAddress }}
                </n-tag>
              </n-td>
            </n-tr>
            <n-tr>
              <n-td>状态</n-td>
              <n-td>
                <n-tag :type="isChanged ? 'warning' : 'success'">
                  {{ isChanged ? '已修改' : '原始地址' }}
                </n-tag>
              </n-td>
            </n-tr>
            <n-tr>
              <n-td>重启自动还原</n-td>
              <n-td>
                <div class="flex items-center gap-2">
                  <n-switch v-model:value="autoRestoreEnabled" @update:value="setAutoRestore" />
                  <span>{{ autoRestoreEnabled ? '开启' : '关闭' }}</span>
                </div>
              </n-td>
            </n-tr>
          </n-tbody>
        </n-table>

        <!-- 操作按钮 -->
        <div class="flex mt-4 gap-4">
          <n-button type="primary" :loading="changing" @click="changeMacAddress">
            随机修改MAC地址
          </n-button>
          <n-button :loading="restoring" @click="restoreMacAddress">
            还原MAC地址
          </n-button>
        </div>

        <!-- 错误信息 -->
        <n-alert v-if="error" type="error" :title="error" class="mt-4" />

        <!-- 帮助信息 -->
        <n-collapse class="mt-4">
          <n-collapse-item title="MAC地址说明" name="help">
            <n-space vertical>
              <div>
                <strong>什么是MAC地址？</strong>
                <p>MAC地址是网络设备的唯一标识符，由48位二进制数字组成，通常表示为12个十六进制数字。</p>
              </div>
              <div>
                <strong>为什么要修改MAC地址？</strong>
                <p>修改MAC地址可用于增强隐私保护、绕过基于MAC地址的网络访问控制等。请确保您的操作符合相关法律法规。</p>
              </div>
            </n-space>
          </n-collapse-item>
        </n-collapse>
      </n-spin>
    </n-card>
  </div>
</template>

<style scoped>
.mac-id-container {
  padding: 20px;
  max-width: 800px;
  margin: 0 auto;
}

.mac-card {
  width: 100%;
}

.mt-4 {
  margin-top: 16px;
}

.flex {
  display: flex;
}

.items-center {
  align-items: center;
}

.gap-2 {
  gap: 8px;
}

.gap-4 {
  gap: 16px;
}
</style>
