import type { Component } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import IconKeyboard from '~icons/material-symbols/keyboard-outline-rounded'
import IconEthernet from '~icons/material-symbols/settings-ethernet-rounded'
import IconTouchApp from '~icons/material-symbols/touch-app-outline-rounded'
import MainLayout from '@/components/layout/MainLayout.vue'
import HotkeyView from '@/views/hotkey/HotkeyView.vue'
import KeyboardView from '@/views/keyboard/KeyboardView.vue'
import MacIdView from '@/views/mac-id/MacId.vue'

declare module 'vue-router' {
  interface RouteMeta {
    title?: string
    /** 侧栏导航图标（单色线条，颜色跟随 currentColor） */
    icon?: Component
  }
}

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      component: MainLayout,
      redirect: '/keyboard',
      children: [
        {
          path: 'keyboard',
          name: 'Keyboard',
          component: KeyboardView,
          meta: { title: '改键', icon: IconKeyboard },
        },
        {
          path: 'mac-id',
          name: 'MacId',
          component: MacIdView,
          meta: { title: 'MAC地址', icon: IconEthernet },
        },
        {
          path: 'hotkey',
          name: 'Hotkey',
          component: HotkeyView,
          meta: { title: '按键', icon: IconTouchApp },
        },
      ],
    },
  ],
})

export default router
