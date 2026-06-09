import { createRouter, createWebHistory } from 'vue-router'
import MainLayout from '@/components/layout/MainLayout.vue'
import HotkeyView from '@/views/hotkey/HotkeyView.vue'
import KeyboardView from '@/views/keyboard/KeyboardView.vue'
import MacIdView from '@/views/mac-id/MacId.vue'

declare module 'vue-router' {
  interface RouteMeta {
    title?: string
    sealChar?: string
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
          meta: { title: '改键', sealChar: '改' },
        },
        {
          path: 'mac-id',
          name: 'MacId',
          component: MacIdView,
          meta: { title: 'MAC地址', sealChar: '网' },
        },
        {
          path: 'hotkey',
          name: 'Hotkey',
          component: HotkeyView,
          meta: { title: '按键', sealChar: '按' },
        },
      ],
    },
  ],
})

export default router
