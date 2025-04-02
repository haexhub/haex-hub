<template>
  <UiTooltip
    :tooltip="tooltip ?? label"
    direction="right-end"
  >
    <button
      class="link flex items-center justify-center py-3 hover:text-primary tooltip-toogle bg w-full"
      @click="$emit('click')"
    >
      <Icon
        :name="icon"
        class="size-8"
      />
    </button>
  </UiTooltip>
</template>

<script setup lang="ts">
defineProps<{
  label: string;
  tooltip?: string;
  icon: string;
}>();
</script>
