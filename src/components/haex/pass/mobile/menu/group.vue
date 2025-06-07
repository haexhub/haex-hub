<template>
  <button
    aria-disabled
    class="flex gap-4 w-full"
    @click="$emit('click', group)"
  >
    <Icon
      :name="groupIcon"
      size="24"
      class="shrink-0"
    />
    <p class="w-full flex-1 text-start truncate">
      {{ group.name }}
    </p>

    <Icon
      name="mdi:chevron-right"
      size="24"
    />
  </button>
</template>

<script setup lang="ts">
import type { SelectHaexPasswordsGroups } from '~~/src-tauri/database/schemas/vault'

defineEmits<{ click: [group: SelectHaexPasswordsGroups] }>()

const { group } = defineProps<{
  group: SelectHaexPasswordsGroups
}>()

const groupIcon = computed(() => group.icon ?? 'mdi:folder-outline')
</script>
