<template>
  <NuxtLinkLocale>
    {{ item.title }}
  </NuxtLinkLocale>
</template>

<script setup lang="ts">
import type { SelectHaexPasswordsItems } from '~~/src-tauri/database/schemas/vault'

defineProps<{
  item: SelectHaexPasswordsItems
}>()
</script>
