<template>
  <div class="min-h-screen flex flex-col">
    <header
      class="bg-default/70 backdrop-blur border-b border-accented h-(--ui-header-height) sticky top-0 z-50 flex"
    >
      <div class="px-2 bg-primary rounded-br-xs">
        <UiLogoHaexhub class="p-2 size-12 shrink-0" />
      </div>
      <div
        class="w-full max-w-(--ui-container) px-4 sm:px-6 lg:px-8 mx-auto flex items-center justify-between gap-3 h-full"
      >
        <div class="lg:flex-1 flex items-center gap-1.5 min-w-0">
          <NuxtLinkLocale
            class="link text-base-content link-neutral text-xl font-semibold no-underline flex items-center"
            :to="{ name: 'vaultOverview' }"
          >
            <UiTextGradient class="text-nowrap">
              {{ currentVaultName }}
            </UiTextGradient>
          </NuxtLinkLocale>
        </div>

        <div class="hidden lg:flex"></div>

        <div class="flex items-center justify-end lg:flex-1 gap-1.5">
          <HaexMenuApplications />
          <UiDropdownVault />
        </div>
      </div>
    </header>

    <div class="flex flex-1">
      <!-- <aside
        id="sidebar"
        class="border-r border-accented transition-all shrink-0 sticky top-(--ui-header-height) h-[calc(100vh-var(--ui-header-height))]"
        :class="[!isVisible ? 'w-0' : 'w-16']"
        role="dialog"
        tabindex="-1"
      >
        <ul class="p-0 h-full flex flex-col gap-2">
          <UiSidebarLink
            v-for="item in menu"
            v-bind="item"
            :key="item.id"
          />
          <UiSidebarLink
            v-for="item in extensionLinks"
            :key="item.id"
            v-bind="item"
            icon-type="svg"
          />
        </ul>
      </aside> -->

      <main class="flex-1 bg-elevated">
        <NuxtPage />
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
const { currentVaultName } = storeToRefs(useVaultStore())
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
