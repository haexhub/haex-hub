<template>
  <button :class="cn(
    `relative flex items-center justify-center min-w-28 min-h-10 overflow-hidden outline-2 outline-offset-2 rounded cursor-pointer`, className
  )
    ">
    <span class="btn-content inline-flex size-full items-center justify-center px-4 py-2 gap-2">
      <slot />
    </span>
  </button>
</template>

<script setup lang="ts">
import { cn } from "@/lib/utils";
const { className = "primary" } = defineProps<{ className?: string }>()
</script>