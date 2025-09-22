<script setup lang="tsx">
import type { MenuOption } from 'naive-ui'
import type { Component } from 'vue'
import type { RouteRecordRaw } from 'vue-router'
import { NIcon, useOsTheme } from 'naive-ui'
import { computed, h } from 'vue'
import { RouterLink, useRouter } from 'vue-router'

const router = useRouter()
// 获取系统主题
const osTheme = useOsTheme()

function renderIcon(icon?: Component) {
  if (!icon)
    return null
  return () => h(NIcon, { size: 36 }, { default: () => h(icon, {
  }) })
}

const menuOptions = computed<MenuOption[]>(() => {
  const rootRoute = router.options.routes.find(r => r.path === '/') as RouteRecordRaw
  const routes = rootRoute?.children || []

  return routes.map(r => ({
    label: () =>
      h(
        RouterLink,
        {
          to: {
            name: r.name,
          },
        },
        { default: () => r.meta!.title as string },
      ),
    key: r.name,
    icon: renderIcon(r.meta?.icon as Component),
  } as unknown as MenuOption))
})

// 计算侧边栏的样式类，根据主题切换不同的背景效果
const siderClass = computed(() => {
  const baseClass = 'h-screen backdrop-blur-sm bg-cover bg-center bg-no-repeat'
  return osTheme.value === 'dark'
    ? `${baseClass} bg-[rgba(0,0,0,0.7)] dark:bg-[url('@/assets/pagoda-8018757_1280.jpg')]`
    : `${baseClass} bg-[url('@/assets/pagoda-8018757_1280.jpg')]`
})
</script>

<template>
  <n-layout has-sider>
    <n-layout-sider
      :collapsed-width="64"
      collapse-mode="width"
      collapsed
      :class="siderClass"
    >
      <n-menu
        collapsed
        class="h-full !bg-transparent"
        :options="menuOptions"
        key-field="key"
      />
    </n-layout-sider>
    <n-layout>
      <n-layout-content content-style="padding: 0;">
        <MRicePaper class="h-screen w-full">
          <router-view />
        </MRicePaper>
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>

<style scoped>
:deep(.n-menu-item-content) {
  padding-left: 20px !important;
}
</style>
