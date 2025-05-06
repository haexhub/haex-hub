<template>
  <div class="bg-blue-200 h-full">aaaaa</div>
</template>

<script setup lang="ts">
definePageMeta({
  name: "vaultOverview",
});

const extensionStore = useExtensionsStore();

onMounted(async () => {
  await extensionStore.loadExtensionsAsync();
});
</script>
