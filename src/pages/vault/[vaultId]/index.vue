<template>
  <UPage>
    <div class="h-screen bg-amber-300 flex-1 flex-wrap">
      aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    </div>
    <div class="h-screen bg-teal-300 flex-1">
      abbbbbbbbbbbbbbbbbbbbb availableThemes:{{ uiStore.availableThemes }}
    </div>
  </UPage>
</template>

<script setup lang="ts">
definePageMeta({
  name: 'vaultOverview',
})

const uiStore = useUiStore()
</script>
