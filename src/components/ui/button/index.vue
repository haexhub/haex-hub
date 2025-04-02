<template>
  <button
    class="btn join-item"
    :class="{
      'btn-sm':
        currentScreenSize === 'sm' ||
        currentScreenSize === '' ||
        currentScreenSize === 'xs',
    }"
    :type
  >
    <slot />
  </button>
</template>

<script setup lang="ts">
const { currentScreenSize } = storeToRefs(useUiStore());

defineProps({
  type: {
    type: String as PropType<'reset' | 'submit' | 'button'>,
    default: 'button',
  },
});
</script>
