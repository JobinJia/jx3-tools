<script setup lang="ts">
import type { RouteRecordRaw } from 'vue-router'
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useTheme } from '@/composables/useTheme'
import { version } from '../../../package.json'

const router = useRouter()
const route = useRoute()
const { mode, cycleMode } = useTheme()

interface NavItem {
  name: string
  title: string
  sealChar: string
}

const navItems = computed<NavItem[]>(() => {
  const rootRoute = router.options.routes.find(r => r.path === '/') as RouteRecordRaw
  return (rootRoute?.children || []).map(r => ({
    name: String(r.name),
    title: r.meta?.title ?? '',
    sealChar: r.meta?.sealChar ?? '',
  }))
})

const themeIcon = computed(() => (mode.value === 'system' ? '◐' : mode.value === 'light' ? '☀' : '☾'))
const themeTitle = computed(() => (mode.value === 'system' ? '跟随系统' : mode.value === 'light' ? '浅色' : '深色'))
</script>

<template>
  <div class="flex h-screen w-full">
    <aside class="sider">
      <nav class="flex flex-col items-center gap-3">
        <n-tooltip v-for="item in navItems" :key="item.name" placement="right">
          <template #trigger>
            <button
              class="seal-nav"
              :class="{ active: route.name === item.name }"
              :aria-label="item.title"
              @click="router.push({ name: item.name })"
            >
              {{ item.sealChar }}
            </button>
          </template>
          {{ item.title }}
        </n-tooltip>
      </nav>
      <div class="mt-auto flex flex-col items-center gap-2">
        <n-tooltip placement="right">
          <template #trigger>
            <button class="theme-toggle" @click="cycleMode">
              {{ themeIcon }}
            </button>
          </template>
          主题：{{ themeTitle }}
        </n-tooltip>
        <span class="version">v{{ version }}</span>
      </div>
    </aside>
    <main class="flex-1 min-w-0 overflow-y-auto paper-bg">
      <router-view />
    </main>
  </div>
</template>

<style scoped>
.sider {
  width: 58px;
  flex-shrink: 0;
  background: var(--sider-bg);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px 0 12px;
}

.seal-nav {
  width: 36px;
  height: 36px;
  border-radius: 5px;
  cursor: pointer;
  font-family: 'Songti SC', 'STSong', 'SimSun', serif;
  font-size: 17px;
  color: var(--sider-text);
  background: transparent;
  border: 1px solid var(--sider-line);
  transition:
    color 0.2s,
    border-color 0.2s,
    background 0.2s;
}

.seal-nav:hover {
  color: var(--sider-active-text);
  border-color: rgba(245, 239, 226, 0.4);
}

.seal-nav.active {
  background: var(--cinnabar);
  color: var(--sider-active-text);
  border-color: transparent;
  box-shadow: inset 0 0 0 1px rgba(245, 239, 226, 0.25);
}

.theme-toggle {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 1px solid var(--sider-line);
  background: transparent;
  color: var(--sider-text);
  cursor: pointer;
  font-size: 13px;
}

.theme-toggle:hover {
  color: var(--sider-active-text);
}

.seal-nav:focus-visible,
.theme-toggle:focus-visible {
  outline: 2px solid var(--sider-active-text);
  outline-offset: 2px;
}

.version {
  font-size: 9px;
  color: var(--sider-text);
  opacity: 0.55;
}
</style>
