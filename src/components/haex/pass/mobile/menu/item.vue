<template>
  <button
    class="flex gap-4 w-full px-4 py-2"
    :style="{ color: menuItem.color ?? '' }"
    @click="$emit('click', menuItem)"
  >
    <Icon
      :name="menuIcon"
      size="24"
      class="shrink-0"
    />
    <p class="w-full flex-1 text-start truncate font-bold">
      {{ menuItem?.name }}
    </p>

    <Icon
      v-if="menuItem.type === 'group'"
      name="mdi:chevron-right"
      size="24"
      class="text-base-content"
    />
  </button>
</template>

<script setup lang="ts">
import type { IPasswordMenuItem } from './types'

defineEmits<{ click: [group?: IPasswordMenuItem] }>()

const menuItem = defineProps<IPasswordMenuItem>()

const menuIcon = computed(() =>
  menuItem?.icon
    ? menuItem.icon
    : menuItem.type === 'group'
      ? 'mdi:folder-outline'
      : 'mdi:key-outline',
)
</script>
