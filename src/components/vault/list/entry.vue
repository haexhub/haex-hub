<template>
  <UiListButton
    v-if="entry"
    :key="entry.id"
    @click="navigateToEntryAsync(entry.id)"
    class="text-base-content"
  >
    <div class="flex items-center gap-3">
      <div class="w-8">
        <Icon
          v-if="entry.icon || groupIcon"
          :name="entry.icon || groupIcon!"
        />
      </div>
      <div class="flex flex-col items-start">
        <div v-if="!entry.title && !entry.username && !entry.url">
          {{ entry.id }}
        </div>
        <div class="font-semibold">
          {{ entry.title }}
        </div>
        <span class="text-sm">
          {{ entry.username }}
        </span>
        <span class="text-sm">
          {{ entry.url }}
        </span>
      </div>
    </div>
  </UiListButton>
</template>

<script setup lang="ts">
import type { SelectVaultEntry } from '~/database/schemas/vault';

defineProps({
  entry: Object as PropType<SelectVaultEntry>,
  groupIcon: [String, null],
});

const { navigateToEntryAsync } = useVaultEntryStore();
</script>
