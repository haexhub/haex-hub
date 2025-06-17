<template>
  <div class="h-full w-full">
    <NuxtLayout name="app">
      <NuxtPage />
    </NuxtLayout>

    <UiDialog v-model:open="showInstanceDialog">
      <div>
        Das scheint das erste Mal zu sein, dass du auf diesem Gerät diese Vault
        öffnest. Bitte gib diesem Gerät einen Namen
      </div>
    </UiDialog>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  middleware: 'database',
})

const showInstanceDialog = ref(false)

const { readNotificationsAsync } = useNotificationStore()
const { isFirstTimeAsync } = useVaultInstanceStore()
const { loadExtensionsAsync } = useExtensionsStore()

onMounted(async () => {
  await loadExtensionsAsync()
  await readNotificationsAsync()

  if (await isFirstTimeAsync()) {
    showInstanceDialog.value = true
  }
})

onMounted(() => {})
</script>
