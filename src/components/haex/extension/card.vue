<template>
  <div class="card">
    <slot name="image" />

    <div class="absolute top-2 right-2">
      <UiButton class="btn-error btn-outline btn-sm btn-square" @click="$emit('remove')">
        <Icon name="mdi:trash" />
      </UiButton>
    </div>

    <div class="card-header">
      <div v-if="$slots.title || name">
        <div class="flex justify-start gap-4">
          <div v-html="icon" class="shrink-0 size-10" />
          <h5 v-if="name" class="card-title m-0 my-auto">
            {{ name }}
          </h5>
        </div>
      </div>
      <div class="text-base-content/50">{{ manifest }}</div>
    </div>

    <div class="card-body">
      <slot />
      <div class="card-actions" v-if="$slots.action">
        <slot name="action" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { IHaexHubExtension } from "~/types/haexhub";
const emit = defineEmits(["close", "submit", "remove"]);

defineProps<IHaexHubExtension>();

const { escape, enter } = useMagicKeys();

watchEffect(async () => {
  if (escape.value) {
    await nextTick();
    emit("close");
  }
});

watchEffect(async () => {
  if (enter.value) {
    await nextTick();
    emit("submit");
  }
});
</script>
