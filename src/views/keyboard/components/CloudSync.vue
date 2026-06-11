<script setup lang="ts">
import type { CloudConfig, UserSelect } from '@/types'
import { computed, ref, watch } from 'vue'
import { cloudProgressPercent, useCloud } from '@/composables/useCloud'
import { useKeyboard } from '@/composables/useKeyboard'

const props = defineProps<{
  userSelect: UserSelect
}>()

const show = defineModel<boolean>('show', { required: true })

const {
  config,
  roles,
  testing,
  saving,
  listing,
  uploading,
  downloading,
  progress,
  loadConfig,
  testConnection,
  saveConfig,
  refreshRoles,
  uploadAll,
  downloadRole,
} = useCloud()
const { basePath, loadTree } = useKeyboard()

const editing = ref(false)
const form = ref<CloudConfig>({
  serverUrl: 'https://dav.jianguoyun.com/dav/',
  username: '',
  appPassword: '',
})

const bound = computed(() => !!config.value && !editing.value)
const formReady = computed(() =>
  form.value.serverUrl.trim() !== '' && form.value.username.trim() !== '' && form.value.appPassword.trim() !== '')

watch(show, async (visible) => {
  if (!visible)
    return
  await loadConfig()
  if (config.value) {
    form.value = { ...config.value }
    editing.value = false
    await refreshRoles()
  } else {
    editing.value = true
  }
})

async function handleSave() {
  if (await saveConfig(form.value)) {
    editing.value = false
    await refreshRoles()
  }
}

function rolePath(relative: string): string {
  return `${basePath.value}/${relative}`
}

async function handleUpload() {
  if (!basePath.value)
    return
  await uploadAll(basePath.value)
}

async function handleDownload(key: string) {
  if (!props.userSelect.targetPath)
    return
  if (await downloadRole(key, rolePath(props.userSelect.targetPath)))
    await loadTree()
}

function formatTime(seconds: number): string {
  const date = new Date(seconds * 1000)
  const pad = (n: number) => `${n}`.padStart(2, '0')
  const day = `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`
  return `${day} ${pad(date.getHours())}:${pad(date.getMinutes())}`
}
</script>

<template>
  <n-modal v-model:show="show" preset="card" title="云同步" style="width: 560px">
    <!-- 账号绑定 -->
    <div class="section">
      <div class="section-title">
        网盘账号（WebDAV）
      </div>
      <template v-if="bound">
        <div class="flex items-center gap-2 text-xs">
          <span class="bound-chip">✓ {{ config!.username }}</span>
          <span class="truncate" style="color: var(--ink-muted)">{{ config!.serverUrl }}</span>
          <a class="edit-link ml-auto" @click="editing = true">修改</a>
        </div>
      </template>
      <template v-else>
        <div class="flex flex-col gap-2">
          <n-input v-model:value="form.serverUrl" size="small" placeholder="服务器地址，坚果云为 https://dav.jianguoyun.com/dav/" />
          <n-input v-model:value="form.username" size="small" placeholder="账号（坚果云为注册邮箱）" />
          <n-input
            v-model:value="form.appPassword"
            size="small"
            type="password"
            show-password-on="click"
            placeholder="应用密码（不是登录密码）"
          />
          <div class="text-[11px] leading-relaxed" style="color: var(--ink-muted)">
            坚果云用户：网页端登录 → 右上角头像 → 账户信息 → 安全选项 → 第三方应用管理 →
            添加应用密码。免费账户每月 1GB 上传流量，键位包只有几十 KB，完全够用。
          </div>
          <div class="flex items-center gap-2">
            <n-button size="small" :loading="testing" :disabled="!formReady" @click="testConnection(form)">
              测试连接
            </n-button>
            <n-button size="small" type="primary" :loading="saving" :disabled="!formReady" @click="handleSave">
              保存
            </n-button>
            <n-button v-if="config" size="small" quaternary @click="editing = false">
              取消
            </n-button>
          </div>
        </div>
      </template>
    </div>

    <template v-if="bound">
      <!-- 进度条：上传/下载期间显示 -->
      <div v-if="progress" class="section">
        <div class="mb-1.5 flex items-center justify-between text-xs">
          <span style="color: var(--ink)">{{ progress.label }}</span>
          <span v-if="progress.total > 0" style="color: var(--ink-muted)">
            {{ progress.current }}/{{ progress.total }}
          </span>
        </div>
        <n-progress
          type="line"
          :percentage="cloudProgressPercent(progress)"
          :height="8"
          :border-radius="4"
          processing
        />
      </div>

      <!-- 上传 -->
      <div class="section">
        <div class="section-title">
          上传到云端
        </div>
        <div class="flex items-center gap-2 text-xs">
          <span style="color: var(--ink-muted)">
            一键把 userdata 下<b>所有角色</b>的键位 + 插件配置打包上传，无需选择（同名角色覆盖云端旧档）
          </span>
          <n-button
            size="small"
            type="primary"
            class="ml-auto flex-shrink-0"
            :loading="uploading"
            :disabled="!basePath"
            @click="handleUpload"
          >
            {{ uploading ? '上传中…' : '全部上传' }}
          </n-button>
        </div>
      </div>

      <!-- 云端列表 -->
      <div class="section">
        <div class="section-title flex items-center">
          云端存档
          <a class="edit-link ml-auto" @click="refreshRoles">⟳ 刷新</a>
        </div>
        <n-spin :show="listing" size="small">
          <div v-if="roles.length === 0" class="py-3 text-center text-xs" style="color: var(--ink-muted)">
            云端还没有存档，先上传一个角色吧
          </div>
          <div v-else class="role-list">
            <div v-for="role in roles" :key="role.key" class="role-row">
              <div class="min-w-0 flex-1">
                <div class="truncate text-xs" style="color: var(--ink)">
                  {{ role.name }}
                  <span style="color: var(--ink-muted)">· {{ role.server }}</span>
                </div>
                <div class="text-[10px]" style="color: var(--ink-muted)">
                  {{ formatTime(role.uploadedAt) }} · {{ role.device }}
                  · 键位{{ role.pluginsFile ? ' + 插件配置' : '' }}
                </div>
              </div>
              <n-tooltip trigger="hover" :disabled="!!userSelect.target">
                <template #trigger>
                  <n-button
                    size="tiny"
                    :loading="downloading"
                    :disabled="!userSelect.targetPath"
                    @click="handleDownload(role.key)"
                  >
                    下载到{{ userSelect.target ? `「${userSelect.target}」` : '…' }}
                  </n-button>
                </template>
                先在改键页选择目标角色
              </n-tooltip>
            </div>
          </div>
        </n-spin>
        <div class="mt-2 text-[10px]" style="color: var(--ink-muted)">
          下载会整体替换目标角色的键位与插件设置（先关闭游戏）；目标角色需用插件登录过一次，插件配置才能落位。
        </div>
      </div>
    </template>
  </n-modal>
</template>

<style scoped>
.section {
  padding: 10px 0;
}

.section + .section {
  border-top: 1px solid var(--line-soft);
}

.section-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--ink);
  margin-bottom: 8px;
}

.bound-chip {
  color: var(--bamboo);
  border: 1px solid var(--bamboo);
  border-radius: 5px;
  padding: 3px 8px;
  background: transparent;
  white-space: nowrap;
}

.edit-link {
  font-size: 11px;
  color: var(--indigo);
  cursor: pointer;
}

.role-list {
  max-height: 220px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.role-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 8px;
  border: 1px solid var(--line-soft);
  border-radius: 6px;
}
</style>
