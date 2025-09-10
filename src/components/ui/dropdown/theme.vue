<template>
  <UDropdownMenu :items>
    <UButton :icon="currentTheme?.icon" />
  </UDropdownMenu>
</template>

<script setup lang="ts">
import type { DropdownMenuItem } from '@nuxt/ui'

const { availableThemes, currentTheme } = storeToRefs(useUiStore())

const emit = defineEmits<{ select: [string] }>()

watchImmediate(availableThemes, () =>
  console.log('availableThemes', availableThemes),
)

const items = computed<DropdownMenuItem[]>(() =>
  availableThemes?.value.map((theme) => ({
    ...theme,
    onSelect: () => emit('select', theme.value),
  })),
)
</script>
