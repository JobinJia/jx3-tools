<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { onMounted, ref } from 'vue'

// 创建消息实例
const message = useMessage()

// 存储MAC地址信息
const originalMacAddress = ref<string>('')
const currentMacAddress = ref<string>('')
const isLoading = ref<boolean>(false)
const isChanging = ref<boolean>(false)
const isRestoring = ref<boolean>(false)
const errorMessage = ref<string>('')
const autoRestoreOnReboot = ref<boolean>(true) // 默认打开

// 计算当前状态
const isMacAddressChanged = ref(false)

// 获取原始MAC地址
async function fetchOriginalMacAddress() {
  isLoading.value = true
  errorMessage.value = ''

  try {
    // 调用Tauri后端获取MAC地址
    const macAddress = await invoke('get_mac_address')
    originalMacAddress.value = macAddress as string
    currentMacAddress.value = macAddress as string
    isMacAddressChanged.value = originalMacAddress.value !== currentMacAddress.value

    // 获取自动还原设置
    try {
      const autoRestore = await invoke('get_auto_restore_setting')
      autoRestoreOnReboot.value = autoRestore as boolean
    } catch (error) {
      console.error('获取自动还原设置失败:', error)
      // 默认为true
      autoRestoreOnReboot.value = true
      // 设置为true
      updateAutoRestoreSetting(true)
    }
  } catch (error) {
    console.error('获取MAC地址失败:', error)
    errorMessage.value = `获取MAC地址失败: ${error}`
    message.error('获取MAC地址失败')
    // 模拟数据，实际应用中应删除
    originalMacAddress.value = '00:1A:2B:3C:4D:5E'
    currentMacAddress.value = '00:1A:2B:3C:4D:5E'
    isMacAddressChanged.value = false
  } finally {
    isLoading.value = false
  }
}

// 生成随机MAC地址
function generateRandomMacAddress(): string {
  const hexDigits = '0123456789ABCDEF'
  let macAddress = ''

  for (let i = 0; i < 6; i++) {
    let part = ''
    for (let j = 0; j < 2; j++) {
      part += hexDigits.charAt(Math.floor(Math.random() * hexDigits.length))
    }
    macAddress += (i === 0 ? '' : ':') + part
  }

  return macAddress
}

// 修改MAC地址
async function changeMacAddress() {
  isChanging.value = true
  errorMessage.value = ''

  try {
    const newMacAddress = generateRandomMacAddress()

    // 调用Tauri后端修改MAC地址
    await invoke('change_mac_address', { macAddress: newMacAddress })
    message.success('MAC地址修改成功')

    // 更新当前MAC地址
    currentMacAddress.value = newMacAddress
    isMacAddressChanged.value = originalMacAddress.value !== currentMacAddress.value
  } catch (error) {
    console.error('修改MAC地址失败:', error)
    errorMessage.value = `修改MAC地址失败: ${error}`
    message.error('修改MAC地址失败')
  } finally {
    isChanging.value = false
  }
}

// 还原MAC地址
async function restoreMacAddress() {
  isRestoring.value = true
  errorMessage.value = ''

  try {
    // 调用Tauri后端还原MAC地址
    await invoke('restore_mac_cmd')
    message.success('MAC地址已还原')

    // 重新获取MAC地址
    const macAddress = await invoke('get_mac_address')
    currentMacAddress.value = macAddress as string
    isMacAddressChanged.value = originalMacAddress.value !== currentMacAddress.value
  } catch (error) {
    console.error('还原MAC地址失败:', error)
    errorMessage.value = `还原MAC地址失败: ${error}`
    message.error('还原MAC地址失败')
  } finally {
    isRestoring.value = false
  }
}

// 更新自动还原设置
async function updateAutoRestoreSetting(value: boolean) {
  try {
    await invoke('set_auto_restore_setting', { autoRestore: value })
    message.success(value ? '已开启重启自动还原' : '已关闭重启自动还原')
    autoRestoreOnReboot.value = value
  } catch (error) {
    console.error('更新自动还原设置失败:', error)
    message.error('更新设置失败')
    // 恢复原值
    autoRestoreOnReboot.value = !value
  }
}

// 组件挂载时获取MAC地址
onMounted(() => {
  fetchOriginalMacAddress()
})
</script>

<template>
  <div class="mac-id-container">
    <n-card title="MAC地址管理" class="mac-card">
      <n-spin :show="isLoading">
        <!-- 简化的表格 -->
        <n-table :bordered="true" :single-line="false">
          <n-tbody>
            <n-tr>
              <n-td>原始MAC地址</n-td>
              <n-td>
                <n-tag type="primary">
                  {{ originalMacAddress }}
                </n-tag>
              </n-td>
            </n-tr>
            <n-tr>
              <n-td>当前MAC地址</n-td>
              <n-td>
                <n-tag :type="isMacAddressChanged ? 'warning' : 'success'">
                  {{ currentMacAddress }}
                </n-tag>
              </n-td>
            </n-tr>
            <n-tr>
              <n-td>状态</n-td>
              <n-td>
                <n-tag :type="isMacAddressChanged ? 'warning' : 'success'">
                  {{ isMacAddressChanged ? '已修改' : '原始地址' }}
                </n-tag>
              </n-td>
            </n-tr>
            <n-tr>
              <n-td>重启自动还原</n-td>
              <n-td>
                <div class="flex items-center gap-2">
                  <n-switch v-model:value="autoRestoreOnReboot" @update:value="updateAutoRestoreSetting" />
                  <span>{{ autoRestoreOnReboot ? '开启' : '关闭' }}</span>
                </div>
              </n-td>
            </n-tr>
          </n-tbody>
        </n-table>

        <!-- 操作按钮 -->
        <div class="flex mt-4 gap-4">
          <n-button type="primary" :loading="isChanging" @click="changeMacAddress">
            随机修改MAC地址
          </n-button>
          <n-button :loading="isRestoring" @click="restoreMacAddress">
            还原MAC地址
          </n-button>
        </div>

        <!-- 错误信息 -->
        <n-alert v-if="errorMessage" type="error" :title="errorMessage" class="mt-4" />

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
