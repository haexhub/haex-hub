<template>
  <button
    class="btn join-item pointer-events-auto"
    :type
  >
    <UiTooltip
      :tooltip
      v-if="tooltip"
    >
      <slot />
    </UiTooltip>

    <slot v-else />
  </button>
</template>

<script setup lang="ts">
const { type = 'button' } = defineProps<{
  type?: 'reset' | 'submit' | 'button'
  tooltip?: string
}>()
</script>
