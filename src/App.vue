<script setup lang="ts">
import { darkTheme, NConfigProvider, NMessageProvider, useOsTheme } from 'naive-ui'

// 获取系统主题
const osTheme = useOsTheme()
</script>

<template>
  <m-rice-paper class="h-screen w-full">
    <NConfigProvider :theme="osTheme === 'dark' ? darkTheme : null">
      <NMessageProvider>
        <router-view />
      </NMessageProvider>
    </NConfigProvider>
  </m-rice-paper>
</template>
