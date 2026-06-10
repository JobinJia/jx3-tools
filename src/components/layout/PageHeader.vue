<script setup lang="ts">
defineProps<{
  title: string
  description?: string
}>()
</script>

<template>
  <div class="mb-2">
    <div class="flex items-center justify-between">
      <div class="min-w-0 flex items-baseline gap-2.5">
        <div class="page-title flex-shrink-0">
          {{ title }} <span class="seal-dot ml-0.5 align-middle" />
        </div>
        <div v-if="description" class="truncate text-xs" style="color: var(--ink-muted)">
          {{ description }}
        </div>
      </div>
      <div v-if="$slots.extra" class="flex flex-shrink-0 items-center">
        <slot name="extra" />
      </div>
    </div>
    <div class="ink-line mt-1.5" />
  </div>
</template>
