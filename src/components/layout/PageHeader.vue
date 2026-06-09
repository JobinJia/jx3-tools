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
          {{ title }} <span class="seal-dot ml-0.5 align-middle" />
        </div>
        <div v-if="description" class="mt-1 text-xs" style="color: var(--ink-muted)">
          {{ description }}
        </div>
      </div>
      <div v-if="$slots.extra" class="flex items-center">
        <slot name="extra" />
      </div>
    </div>
    <div class="ink-line mt-2.5" />
  </div>
</template>
