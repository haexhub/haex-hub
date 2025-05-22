<template>
  <button class="btn join-item" :type>
    <slot />
  </button>
</template>

<script setup lang="ts">
defineProps({
  type: {
    type: String as PropType<"reset" | "submit" | "button">,
    default: "button",
  },
});
</script>