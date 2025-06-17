<template>
  <div>
    aaaa
    <div class="h-full text-base-content flex bg-base-200">
      <HaexExtensionCard
        v-for="extension in extensionStore.availableExtensions"
        v-bind="extension"
        :key="extension.id"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  name: 'vaultOverview',
})

const extensionStore = useExtensionsStore()
</script>
