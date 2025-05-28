<template>
  <div class="h-full text-base-content flex bg-base-200 p-4">
    <HaexExtensionCard
      v-for="extension in extensionStore.availableExtensions"
      v-bind="extension"
      :key="extension.id"
    />
  </div>
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
