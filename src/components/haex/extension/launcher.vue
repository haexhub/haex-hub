<template>
  <UPopover v-model:open="open">
    <UButton
      icon="material-symbols:apps"
      color="neutral"
      variant="outline"
      v-bind="$attrs"
      size="xl"
    />

    <template #content>
      <ul class="p-4 max-h-96 grid grid-cols-3 gap-2 overflow-scroll">
        <!-- Enabled Extensions -->
        <UiButton
          v-for="extension in enabledExtensions"
          :key="extension.id"
          square
          size="xl"
          variant="ghost"
          :ui="{
            base: 'size-24 flex flex-wrap text-sm items-center justify-center overflow-visible',
            leadingIcon: 'size-10',
            label: 'w-full',
          }"
          :icon="extension.icon || 'i-heroicons-puzzle-piece-solid'"
          :label="extension.name"
          :tooltip="extension.name"
          @click="openExtension(extension.id)"
        />

        <!-- Disabled Extensions (grayed out) -->
        <UiButton
          v-for="extension in disabledExtensions"
          :key="extension.id"
          square
          size="xl"
          variant="ghost"
          :disabled="true"
          :ui="{
            base: 'size-24 flex flex-wrap text-sm items-center justify-center overflow-visible opacity-40',
            leadingIcon: 'size-10',
            label: 'w-full',
          }"
          :icon="extension.icon || 'i-heroicons-puzzle-piece-solid'"
          :label="extension.name"
          :tooltip="`${extension.name} (${t('disabled')})`"
        />

        <!-- Marketplace Button (always at the end) -->
        <UiButton
          square
          size="xl"
          variant="soft"
          color="primary"
          :ui="{
            base: 'size-24 flex flex-wrap text-sm items-center justify-center overflow-visible',
            leadingIcon: 'size-10',
            label: 'w-full',
          }"
          icon="i-heroicons-plus-circle"
          :label="t('marketplace')"
          :tooltip="t('marketplace')"
          @click="openMarketplace"
        />
      </ul>
    </template>
  </UPopover>
</template>

<script setup lang="ts">
const extensionStore = useExtensionsStore()
const router = useRouter()
const route = useRoute()
const localePath = useLocalePath()
const { t } = useI18n()

const open = ref(false)

// Enabled extensions first
const enabledExtensions = computed(() => {
  return extensionStore.availableExtensions.filter((ext) => ext.enabled)
})

// Disabled extensions last
const disabledExtensions = computed(() => {
  return extensionStore.availableExtensions.filter((ext) => !ext.enabled)
})

const openExtension = (extensionId: string) => {
  router.push(
    localePath({
      name: 'haexExtension',
      params: {
        vaultId: route.params.vaultId,
        extensionId,
      },
    }),
  )
  open.value = false
}

const openMarketplace = () => {
  router.push(
    localePath({
      name: 'extensionOverview',
      params: {
        vaultId: route.params.vaultId,
      },
    }),
  )
  open.value = false
}
</script>

<i18n lang="yaml">
de:
  disabled: Deaktiviert
  marketplace: Marketplace

en:
  disabled: Disabled
  marketplace: Marketplace
</i18n>
