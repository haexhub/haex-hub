<template>
  <div class="h-full text-base-content"></div>
</template>

<script setup lang="ts">
definePageMeta({
  name: 'vaultOverview',
})

const extensionStore = useExtensionsStore()

onMounted(async () => {
  await extensionStore.loadExtensionsAsync()
})
</script>
