<template>
  <div
    v-if="modelValue?.length"
    class="flex flex-col gap-2"
  >
    <h5
      v-if="title"
      class="text-sm font-semibold text-gray-700 dark:text-gray-300"
    >
      {{ title }}
    </h5>
    <div class="flex flex-col gap-2">
      <HaexExtensionPermissionItem
        v-for="(perm, index) in modelValue"
        :key="perm.target"
        v-model="modelValue[index]!"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { PermissionEntry } from '~~/src-tauri/bindings/PermissionEntry'

defineProps<{
  title?: string
}>()

const modelValue = defineModel<PermissionEntry[]>({ default: () => [] })
</script>
