<template>
  <UiCard
    :title
    :icon
  >
    <slot />
  </UiCard>
</template>

<script setup lang="ts">
defineProps<{ title: string; icon?: string }>()
</script>
