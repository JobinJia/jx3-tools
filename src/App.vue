<script setup lang="ts">
import { NConfigProvider, NMessageProvider } from 'naive-ui'
import { computed } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { darkOverrides, lightOverrides } from '@/theme/naive'

const { naiveTheme, isDark } = useTheme()
const overrides = computed(() => (isDark.value ? darkOverrides : lightOverrides))
</script>

<template>
  <NConfigProvider :theme="naiveTheme" :theme-overrides="overrides" class="h-screen w-full">
    <NMessageProvider>
      <router-view />
    </NMessageProvider>
  </NConfigProvider>
</template>
