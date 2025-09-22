import { createRouter, createWebHistory } from 'vue-router'
import FluentEmojiFlatAngryFace from '~icons/fluent-emoji-flat/angry-face'
import FluentEmojiFlatAnguishedFace from '~icons/fluent-emoji-flat/anguished-face'
import FluentEmojiFlatAstonishedFace from '~icons/fluent-emoji-flat/astonished-face'
import MainLayout from '@/components/layout/MainLayout.vue'

// import NotoV1JapaneseProhibitedButton from '~icons/noto-v1/japanese-prohibited-button'
// import NotoV1JapaneseSecretButton from '~icons/noto-v1/japanese-secret-button'

import HotkeyView from '@/views/hotkey/HotkeyView.vue'
import KeyboardView from '@/views/keyboard/KeyboardView.vue'
import MacIdView from '@/views/mac-id/MacId.vue'

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
          meta: {
            title: '改键',
            icon: FluentEmojiFlatAstonishedFace,
          },
        },
        {
          path: 'mac-id',
          name: 'MacId',
          component: MacIdView,
          meta: {
            title: 'Mac地址',
            icon: FluentEmojiFlatAngryFace,
          },
        },
        {
          path: 'hotkey',
          name: 'Hotkey',
          component: HotkeyView,
          meta: {
            title: '按键',
            icon: FluentEmojiFlatAnguishedFace,
          },
        },
      ],
    },
  ],
})

export default router
