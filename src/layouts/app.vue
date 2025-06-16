<template>
  <div class="h-full w-full flex flex-col overflow-hidden">
    <nav
      class="navbar rounded-b max-sm:shadow border-b border-base-content/25 sm:z-20 relative px-2 py-0 sm:py-2"
    >
      <UiTooltip :tooltip="isVisible ? t('sidebar.close') : t('sidebar.show')">
        <button
          ref="sidebarToogleRef"
          type="button"
          class="btn btn-text btn-square me-2 z-50"
          aria-haspopup="dialog"
          aria-expanded="false"
          aria-controls="sidebar"
          @click="toogleSidebar"
        >
          <Icon
            :name="
              isVisible
                ? 'tabler:layout-sidebar-filled'
                : 'tabler:layout-sidebar'
            "
            size="28"
          />
        </button>
      </UiTooltip>

      <div class="flex flex-1 items-center">
        <NuxtLinkLocale
          class="link text-base-content link-neutral text-xl font-semibold no-underline"
          :to="{ name: 'vaultOverview' }"
        >
          <UiTextGradient class="text-nowrap">
            {{ currentVaultName }}
          </UiTextGradient>
        </NuxtLinkLocale>
      </div>

      <div class="flex items-center gap-4 me-4">
        <HaexMenuNotifications />
        <HaexMenuMain />
      </div>
    </nav>

    <div class="flex h-full w-full overflow-hidden">
      <aside
        id="sidebar"
        class="sm:shadow-none transition-all h-full overflow-hidden border-r border-base-300"
        :class="[!isVisible ? 'w-0' : 'w-16']"
        role="dialog"
        tabindex="-1"
      >
        <div class="drawer-body h-full">
          <ul class="menu p-0 h-full rounded-none">
            <HaexSidebarLink
              v-for="item in menu"
              v-bind="item"
              :key="item.id"
            />
            <HaexSidebarLink
              v-for="item in extensionLinks"
              :key="item.id"
              v-bind="item"
              icon-type="svg"
            />
          </ul>
        </div>
      </aside>

      <main class="w-full h-full overflow-auto">
        <NuxtPage />
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
const { t } = useI18n()

const { currentVaultName } = storeToRefs(useVaultStore())

const { menu, isVisible } = storeToRefs(useSidebarStore())

const { extensionLinks } = storeToRefs(useExtensionsStore())

const toogleSidebar = () => {
  isVisible.value = !isVisible.value
}
</script>

<i18n lang="yaml">
de:
  vault:
    close: Vault schließen
  sidebar:
    close: Sidebar ausblenden
    show: Sidebar anzeigen
en:
  vault:
    close: Close vault
  sidebar:
    close: close sidebar
    show: show sidebar
</i18n>
