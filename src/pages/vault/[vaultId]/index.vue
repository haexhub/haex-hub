<template>
  <div class="text-base-content flex flex-col">
    <div class="h-screen bg-amber-300">
      aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    </div>
    <div class="h-screen bg-teal-300">
      abbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
      availableThemes:{{ uiStore.availableThemes }}
    </div>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  name: 'vaultOverview',
})

const uiStore = useUiStore()
</script>
