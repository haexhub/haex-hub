<template>
  <div class="">
    <!-- class="" -->
    <UPageHeader
      as="header"
      :ui="{
        root: [
          'bg-default border-b border-accented sticky top-0 z-50 py-0 px-8',
        ],
        wrapper: [
          'pt-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4',
        ],
      }"
    >
      <template #title>
        <div class="flex items-center">
          <UiLogoHaexhub class="size-12 shrink-0" />

          <NuxtLinkLocale
            class="link text-base-content link-neutral text-xl font-semibold no-underline flex items-center"
            :to="{ name: 'vaultOverview' }"
          >
            <UiTextGradient class="text-nowrap">
              {{ currentVaultName }}
            </UiTextGradient>
          </NuxtLinkLocale>
        </div>
      </template>

      <template #links>
        <HaexExtensionLauncher :block="isSmallScreen" />
        <UiDropdownVault :block="isSmallScreen" />
      </template>
    </UPageHeader>

    <main class="overflow-scroll flex bg-elevated">
      <NuxtPage />
    </main>
  </div>
</template>

<script setup lang="ts">
const { currentVaultName } = storeToRefs(useVaultStore())

const { isSmallScreen } = storeToRefs(useUiStore())
</script>

<i18n lang="yaml">
de:
  vault:
    close: Vault schließen

  sidebar:
    close: Sidebar ausblenden
    show: Sidebar anzeigen

  search:
    label: Suche
en:
  vault:
    close: Close vault
  sidebar:
    close: close sidebar
    show: show sidebar

  search:
    label: Search
</i18n>
