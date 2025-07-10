<template>
  <div class="h-full text-base-content flex bg-base-200">
    <HaexExtensionCard
      v-for="extension in extensionStore.availableExtensions"
      v-bind="extension"
      :key="extension.id"
    />
    <UiButton @click="requesty()">Storage Request</UiButton>
    res: {{ res }}
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  name: 'vaultOverview',
})

const storage = useAndroidStorage()
const extensionStore = useExtensionsStore()

const res = ref()

const requesty = async () => {
  try {
    res.value = await storage.requestStoragePermission()
    res.value += ' wat the fuk'
  } catch (error) {
    res.value = error
  }
}
</script>
