<script setup lang="ts">
defineProps<{
  title: string
  description?: string
}>()
</script>

<template>
  <div class="mb-3.5">
    <div class="flex items-end justify-between">
      <div>
        <div class="page-title">
          {{ title }} <span class="seal-dot align-middle ml-0.5" />
        </div>
        <div v-if="description" class="text-xs mt-1" style="color: var(--ink-muted)">
          {{ description }}
        </div>
      </div>
      <div class="flex items-center">
        <slot name="extra" />
      </div>
    </div>
    <div class="ink-line mt-2.5" />
  </div>
</template>
